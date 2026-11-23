# D1202 vs APK protocol comparison (target: D1204 / A3956)

Scope: research only. Repository root: `E:\User\Documents\Repositories\SoundCore-Desktop`.
All paths below are relative to that root. Line numbers are from the checked-out working tree.
APK facts supplied by the requester are treated as *external observations*, not as source code;
where they conflict with the vendored OpenSCQ30 source, the conflict is called out explicitly.

APK facts used for comparison:

- SPP UUID `0CF12D31-FAC3-4553-BD80-D6832E7B395C`
- outbound frame `08 EE 00 00 00 <group> <cmd> | u16-BE length | payload | u8 additive checksum`
- inbound frame `09 FF 00 00 00 00 00 | u16-BE length | body | u8 checksum`
- ANC command `06 81`, software EQ command `02 88`
- D1204 is internally A3956 family but shares `D1202CmdService` with D1202C

---

## 1. SPP UUID discovery and use

### 1.1 There is no model-to-UUID derivation

`DeviceModel` is only used to select a driver module; it is never used to compute an RFCOMM UUID:

- `OpenSCQ30/lib/src/devices/device_model.rs:32-75` — enum variants; `SoundcoreD1202` at line 70, `SoundcoreD1202C` at line 71. No `D1204`/`A3956` variant exists.
- `OpenSCQ30/lib/src/devices/device_model.rs:134-135` and `:193-194` — both D1202 and D1202C map to `soundcore::d1202`.
- `OpenSCQ30/lib/src/devices/device_model.rs:77-91` — `device_registry()` passes `*self` (the model) into the registry, but the registry uses it only for database keys and settings identity.

### 1.2 The actual UUID logic (verbatim)

`OpenSCQ30/lib/src/devices/soundcore.rs:40-49`:

```rust
use uuid::{Uuid, uuid};

const RFCOMM_UUID: Uuid = uuid!("00001101-0000-1000-8000-00805f9b34fb");
const VENDOR_RFCOMM_UUID: Uuid = uuid!("0cf12d31-fac3-4553-bd80-d6832e700000");
const VENDOR_RFCOMM_UUID_MASK: Uuid = uuid!("ffffffff-ffff-ffff-ffff-fffffff00000");

fn is_soundcore_vendor_rfcomm_uuid(uuid: &Uuid) -> bool {
    let mask = VENDOR_RFCOMM_UUID_MASK.as_u128();
    uuid.as_u128() & mask == VENDOR_RFCOMM_UUID.as_u128() & mask
}
```

Mask math: `fffffff00000` retains the top 28 bits of the final 48-bit group and clears the low 20 bits.
`d6832e700000 & mask == d6832e700000`; the APK UUID's final group `d6832e7b395c & mask == d6832e700000`.
**Therefore the APK UUID `...D6832E7B395C` already satisfies `is_soundcore_vendor_rfcomm_uuid()` and is selected by the default driver strategy.**
There is no literal `B395C` anywhere in the vendored source (grep for `395C`/`0CF12D31` returns only the masked constant above).

Default strategy, `OpenSCQ30/lib/src/devices/soundcore/common/device.rs:80-102`:

```rust
#[derive(Copy, Clone)]
pub struct SoundcoreDeviceConfig {
    /// Most devices have a checksum at the end of their packets, but for the ones that don't,
    /// this can be set to ChecksumKind::None to disable checksums.
    pub checksum_kind: packet::ChecksumKind,
    pub rfcomm_service_selection_strategy: RfcommServiceSelectionStrategy,
}

impl Default for SoundcoreDeviceConfig {
    fn default() -> Self {
        Self {
            checksum_kind: Default::default(),
            rfcomm_service_selection_strategy: RfcommServiceSelectionStrategy::Dynamic(
                |service_uuids| {
                    service_uuids
                        .into_iter()
                        .find(soundcore::is_soundcore_vendor_rfcomm_uuid)
                        .unwrap_or(soundcore::RFCOMM_UUID)
                },
            ),
        }
    }
}
```

`d1202` uses `soundcore_device!(...)` with the optional config omitted (`OpenSCQ30/lib/src/devices/soundcore/d1202.rs:31-91`), so the macro inserts `Default::default()` (`OpenSCQ30/lib/src/devices/soundcore/common/macros.rs:13-27`).

`RfcommServiceSelectionStrategy` is defined at `OpenSCQ30/lib/src/api/connection.rs:56-60`:

```rust
#[derive(Copy, Clone)]
pub enum RfcommServiceSelectionStrategy {
    Constant(Uuid),
    Dynamic(fn(HashSet<Uuid>) -> Uuid),
}
```

Windows back-end enumeration/selection, `OpenSCQ30/lib/src/connection_backend/windows/rfcomm.rs:168-184`:

```rust
            RfcommServiceSelectionStrategy::Dynamic(select_service) => {
                let services = device.GetRfcommServicesAsync()?.join()?.Services()?;
                let mut services_by_uuid = services
                    .clone()
                    .into_iter()
                    .map(|service| Ok((service.ServiceId()?.Uuid()?.as_uuid(), service)))
                    .collect::<windows::core::Result<HashMap<_, _>>>()?;
                debug!("found RFCOMM services: {:?}", services_by_uuid.keys());
                let uuid = select_service(services_by_uuid.keys().copied().collect());
                debug!("using RFCOMM service: {uuid:?}");
                services_by_uuid
                    .remove(&uuid)
                    .ok_or(connection::Error::DeviceNotFound {
                        source: None,
                        location: Location::caller(),
                    })
            }
```

If a fixed UUID is preferred for a D1204 driver, the existing pattern is a `Constant` config, e.g. `OpenSCQ30/lib/src/devices/soundcore/a3930.rs:95-100`:

```rust
const CONFIG: SoundcoreDeviceConfig = SoundcoreDeviceConfig {
    checksum_kind: ChecksumKind::Suffix,
    rfcomm_service_selection_strategy: RfcommServiceSelectionStrategy::Constant(uuid!(
        "00001101-0000-1000-8000-00805f9b34fb"
    )),
};
```

**Conclusion for D1204:** no UUID code change is required to connect to `...B395C`; optionally pin it with `Constant(uuid!("0cf12d31-fac3-4553-bd80-d6832e7b395c"))` if deterministic service selection is desired.

---

## 2. Packet encoder / decoder, checksum, fragmentation

All framing lives in `common`, not in `d1202`. `d1202` only supplies `Command` pairs and bodies.

### 2.1 Direction bytes — verbatim

`OpenSCQ30/lib/src/devices/soundcore/common/packet.rs:186-205`:

```rust
    pub const fn bytes(&self) -> [u8; 5] {
        match self {
            Self::Outbound => [0x08, 0xee, 0x00, 0x00, 0x00],
            Self::Inbound => [0x09, 0xff, 0x00, 0x00, 0x01],
        }
    }

    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        self,
    ) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], Self, E> {
        move |input| {
            context(
                "packet direction",
                map(tag(self.bytes().as_slice()), |_| self),
            )
            .parse(input)
        }
    }
```

The 2 bytes after the 5-byte direction prefix are the command (`Command([group, cmd])`, `packet.rs:207-221`). So the outbound on-the-wire prefix is `08 EE 00 00 00 <group> <cmd>` — an exact match for the APK fact. The inbound prefix produced by this code is `09 FF 00 00 01 <group> <cmd>` (e.g. state update `09 FF 00 00 01 01 01`); the APK note says `09 FF 00 00 00 00 00`. Byte index 4 differs (`0x01` vs `0x00`) and the APK description appears to fold the command bytes into the "header" as zeros. **Verify against a real D1204 capture before changing `Direction::Inbound::bytes()`; it is shared by every device driver.**

### 2.2 Encoder — verbatim

`OpenSCQ30/lib/src/devices/soundcore/common/packet.rs:127-160`:

```rust
    pub fn bytes(&self, checksum_kind: ChecksumKind) -> Vec<u8> {
        // match makes sure we we have to update this function if another variant is added
        let has_checksum = match checksum_kind {
            ChecksumKind::None => false,
            ChecksumKind::Suffix => true,
        };

        const PACKET_SIZE_LENGTH: usize = 2;
        let checksum_length = if has_checksum { 1 } else { 0 };

        let direction_indicator = D::DIRECTION.bytes();
        let command = self.command.0;

        let length = direction_indicator.len()
            + command.len()
            + PACKET_SIZE_LENGTH
            + self.body.len()
            + checksum_length;

        let mut bytes = Vec::with_capacity(length);

        bytes.extend(
            direction_indicator
                .into_iter()
                .chain(command)
                .chain((length as u16).to_le_bytes())
                .chain(self.body.iter().copied()),
        );
        if has_checksum {
            bytes.push(checksum::calculate_checksum(&bytes));
        }

        bytes
    }
```

Length covers the whole frame (direction + command + length field + body + checksum) and is encoded **little-endian** (`to_le_bytes`). Checksum covers every preceding byte.

Ground-truth test vector, `OpenSCQ30/lib/src/devices/soundcore/common/packet/outbound/request_state.rs:28-35`:

```rust
    #[test]
    fn it_matches_an_example_request_state_packet() {
        const EXPECTED: &[u8] = &[0x08, 0xee, 0x00, 0x00, 0x00, 0x01, 0x01, 0x0a, 0x00, 0x02];
        assert_eq!(
            EXPECTED,
            RequestState::default().to_packet().bytes_with_checksum()
        );
    }
```

`0x0a 0x00` = 10 = 5 + 2 + 2 + 0 + 1. **This directly contradicts the APK's `u16-BE length` note for this protocol family.** Do not switch to big-endian in the shared codec; if D1204 truly sends BE, implement a separate codec/byte-order parameter and validate with a capture.

### 2.3 Decoder — verbatim

`OpenSCQ30/lib/src/devices/soundcore/common/packet.rs:65-109`:

```rust
    /// This makes use of nom's streaming parsers, so Err::Incomplete will be returned if the packet
    /// is not done being read yet.
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        checksum_kind: ChecksumKind,
    ) -> impl FnOnce(&'a [u8]) -> IResult<&'a [u8], Self, E> {
        // match makes sure we we have to update this function if another variant is added
        let has_checksum = match checksum_kind {
            ChecksumKind::None => false,
            ChecksumKind::Suffix => true,
        };
        move |full_input| {
            let (input, (_direction, command, length)) = context(
                "header",
                (
                    D::DIRECTION.take(),
                    Command::take,
                    context("packet length", le_u16),
                ),
            )
            .parse(full_input)?;

            // 5 byte direction, 2 byte command, 2 byte length, 1 byte checksum
            let body_length = length.saturating_sub(5 + 2 + 2 + if has_checksum { 1 } else { 0 });

            let (input, body) = context("body", take(body_length)).parse(input)?;
            let header_and_body = &full_input[..full_input.len() - input.len()];
            let input = if has_checksum {
                let (input, _checksum) = context(
                    "checksum",
                    map_opt(le_u8, |checksum| {
                        if checksum == checksum::calculate_checksum(header_and_body) {
                            Some(checksum)
                        } else {
                            None
                        }
                    }),
                )
                .parse(input)?;
                input
            } else {
                input
            };
            Ok((input, Self::new(command, body.to_vec())))
        }
    }
```

Notes: `le_u16` again; `saturating_sub` tolerates malformed short lengths; a checksum mismatch produces a nom error (no resync inside the parser).

### 2.4 Checksum — verbatim

`OpenSCQ30/lib/src/devices/soundcore/common/packet/checksum.rs:1-5`:

```rust
pub fn calculate_checksum<'a>(checksum_data: impl IntoIterator<Item = &'a u8>) -> u8 {
    checksum_data
        .into_iter()
        .fold(0_u8, |acc, curr| acc.wrapping_add(*curr))
}
```

Additive mod-256 over all frame bytes preceding the checksum. This matches the APK statement. `ChecksumKind` is `None | Suffix` with `Suffix` default (`packet.rs:223-228`); a driver can opt out via `SoundcoreDeviceConfig.checksum_kind`.

### 2.5 Fragmentation / coalescing / recovery — verbatim

`OpenSCQ30/lib/src/devices/soundcore/common/packet/packet_io_controller.rs:57-103`:

```rust
    fn spawn_packet_handler(
        checksum_kind: ChecksumKind,
        packet_queues: Arc<MultiQueue<Command, packet::Inbound>>,
        mut incoming_receiver: mpsc::Receiver<Vec<u8>>,
    ) -> (JoinHandle<()>, mpsc::Receiver<packet::Inbound>) {
        let (outgoing_sender, outgoing_receiver) = mpsc::channel(100);
        let handle = tokio::spawn(async move {
            let mut buffer = Vec::<u8>::new();

            'receive_packet:
            while let Some(mut bytes) = incoming_receiver.recv().await {
                buffer.extend_from_slice(&bytes);
                let mut start_index = 0;
                while start_index < buffer.len() {
                    let (remainder, packet) =
                        match packet::Inbound::take::<VerboseError<_>>(checksum_kind)(&buffer[start_index..]) {
                            Ok(parsed) => parsed,
                            Err(nom::Err::Incomplete(_)) => break,
                            Err(err) => {
                                tracing::warn!("failed to parse packet: {err:?}");
                                tracing::warn!("clearing buffer: {buffer:?}");
                                buffer.clear();
                                continue 'receive_packet;
                            }
                        };

                    tracing::debug!("received packet {packet:?}");
                    let packet_length = buffer.len() - start_index - remainder.len();
                    start_index += packet_length;

                    if !packet_queues.pop(&packet.command, packet.clone()) {
                        match outgoing_sender.send(packet).await {
                            Ok(()) => (),
                            Err(err) => tracing::debug!(
                                "received packet that wasn't an ok, but the channel is closed, so it won't be forwarded: {err:?}"
                            ),
                        }
                    }
                }
                // Reuse bytes allocation for new buffer containing the remaining partial packet
                bytes.clear();
                bytes.extend_from_slice(&buffer[start_index..]);
                buffer = bytes;
            }
        }.instrument(info_span!("packet handler")));
        (handle, outgoing_receiver)
    }
```

Streaming nom parsers mean partial frames are retained (`Incomplete` → break) and multiple frames arriving in one RFCOMM read are split. Parse errors clear the whole buffer. Tests proving these behaviours: `packet_io_controller.rs:320-340` (byte-by-byte fragment), `:342-380` (merged frames), `:382-410` (garbage recovery).

Request/response correlation and retries, `packet_io_controller.rs:109-138`:

```rust
    #[tracing::instrument(skip(self))]
    pub async fn send_with_response(
        &self,
        packet: &packet::Outbound,
    ) -> device::Result<packet::Inbound> {
        let queue_key = packet.command;
        let handle = self.packet_queues.add(queue_key);

        handle.wait_for_start().await;

        let packet_bytes = packet.bytes(self.checksum_kind);
        // retry
        for i in 1..=3 {
            tracing::debug!("sending packet ({i})");
            self.connection.write(&packet_bytes).await?;

            if tokio::time::timeout(Duration::from_millis(500 * i), handle.wait_for_end())
                .await
                .is_ok()
            {
                return Ok(handle.wait_for_value().await);
            }
        }

        self.packet_queues.cancel(&queue_key, handle);

        Err(device::Error::ActionTimedOut {
            action: "resending packet until ack received",
        })
    }
```

Responses are matched by `Command`, serialized per command through `MultiQueue` (`common/packet/multi_queue.rs:22-101`). `send_with_multi_response` (`:143-191`) handles the dual-connections case (0..N responses, 2 s timeout).

### 2.6 APK vs code — framing summary

| Aspect | APK observation | OpenSCQ30 d1202 code | Assessment |
|---|---|---|---|
| outbound prefix | `08 EE 00 00 00 <group> <cmd>` | identical (`packet.rs:189`, `Command.0`) | match |
| inbound prefix | `09 FF 00 00 00 00 00` | `09 FF 00 00 01 <cmd0> <cmd1>` (`packet.rs:190`) | byte 4 differs; verify capture |
| length | `u16-BE` | `le_u16` / `to_le_bytes()` (`packet.rs:81,152`) | **conflict**; verify capture |
| checksum | `u8 additive` | `wrapping_add` over preceding bytes (`checksum.rs:1-5`) | match |
| fragmentation | not stated | buffer + nom streaming, retries, per-command queues | implemented in shared transport |

---

## 3. Command table for d1202

### 3.1 d1202-specific commands (verbatim)

`OpenSCQ30/lib/src/devices/soundcore/d1202/packets/outbound.rs:8-17` (spatial audio):

```rust
pub fn set_spatial_audio(spatial_audio: &d1202::structures::SpatialAudio) -> packet::Outbound {
    packet::Outbound::new(
        packet::Command([16, 129]),
        vec![
            u8::from(spatial_audio.is_enabled),
            0,
            spatial_audio.mode as u8,
        ],
    )
}
```

`[16, 129]` = `10 81` (group 0x10, cmd 0x81); payload `[enabled, 0, mode]`.

`OpenSCQ30/lib/src/devices/soundcore/d1202/packets/outbound.rs:98` (EQ):

```rust
    packet::Outbound::new(packet::Command([3, 135]), body)
```

`[3, 135]` = `03 87` (EQ + custom Hear-ID, 114-byte body).

### 3.2 Shared commands used by the d1202 builder

`OpenSCQ30/lib/src/devices/soundcore/d1202.rs:31-91` registers: state update, sound modes, EQ, buttons, limit-high-volume, dual connections, spatial audio, LDAC, auto power off, touch tone, low battery prompt, TWS status, dual battery, case battery, serial + dual firmware.

| Group.Cmd | Direction | Meaning / body | Definition |
|---|---|---|---|
| `01 01` | out/in | request full state / unsolicited state update | `common/packet/outbound/request_state.rs:9,12-22` (verbatim below); `common/packet/inbound/state.rs:3`; test vector at `request_state.rs:30` |
| `01 02` | in | TWS status update | `common/packet/inbound/tws_status.rs:18` |
| `01 03` | out/in | battery level (left/right/case) | `common/packet/outbound/request_battery_level.rs:13`; `common/packet/inbound/battery_level.rs:20,44` |
| `01 04` | out/in | battery charging flags | `common/packet/outbound/request_battery_charging.rs:13`; `common/packet/inbound/battery_charging.rs:21,47` |
| `01 05` | out/in | serial number + dual firmware version | `common/packet/outbound/request_serial_number_and_firmware_version.rs:9`; `common/packet/inbound/serial_number_and_firmware_version.rs:23` |
| `01 7F` | out/in | request LDAC state / LDAC state update (1 byte) | `common/packet/outbound.rs:52`; `common/packet/inbound/ldac_state.rs:18` |
| `01 83` | out | set touch tone | `common/packet/outbound.rs:44`; registered via `builder.touch_tone()` (`d1202.rs:71`) |
| `01 86` | out | set auto power off (enabled, duration index) | `common/packet/outbound/set_auto_power_off.rs:10,17` |
| `01 FF` | out | set LDAC | `common/packet/outbound.rs:51`; `common/modules/flag.rs:151-160` |
| `03 87` | out | set EQ + custom Hear-ID (d1202 custom encoder) | `d1202/packets/outbound.rs:98` |
| `04 81` | out | set single button action `[side, button_id, action_id]` | `common/packet/outbound/set_button_configuration.rs:17-24` |
| `04 82` | out | reset all button configuration | `set_button_configuration.rs:33-38` |
| `04 83` | out | set button enabled `[side, button_id, enabled]` | `set_button_configuration.rs:51-61` |
| `04 84` | out | set all button configurations (8 packed bytes) | `set_button_configuration.rs:73-83` |
| `06 01` | in | sound modes update (8 bytes) | `common/modules/sound_modes_v2/packet_handler.rs:18`; `common/packet/inbound/sound_modes.rs:18` |
| `06 81` | out | **set sound modes / ANC** (8-byte body) — APK match | `common/packet/outbound/set_sound_modes.rs:9,16`; `common/modules/sound_modes_v2/state_modifier.rs:44-49` |
| `0B 01` | out/in | request dual-connections device list (0..N responses) | `common/packet/outbound/dual_connections.rs:6`; `common/packet/inbound/dual_connections_device.rs:25` |
| `0B 84` | out | enable/disable dual connections | `dual_connections.rs:10` |
| `0B 83` | out | connect device (MAC bytes) | `dual_connections.rs:14` |
| `0B 82` | out | disconnect device (MAC bytes) | `dual_connections.rs:18` |
| `0B 81` | out | pair device (MAC bytes) | `dual_connections.rs:22` |
| `10 82` | out | set low battery prompt | `common/packet/outbound.rs:50`; `builder.low_battery_prompt()` (`d1202.rs:72`) |
| `20 81` | out | set decibel reading refresh rate `[rate]` | `common/packet/outbound/set_limit_high_volume.rs:19-22` |
| `20 82` | out | set limit high volume `[enabled, db_limit]` | `common/packet/outbound/set_limit_high_volume.rs:6-17` |
| `16 81` | out | set spatial audio `[enabled, 0, mode]` | `d1202/packets/outbound.rs:8-17` |

Shared constants verbatim (`common/packet/outbound/request_state.rs:5-22`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RequestState;

impl RequestState {
    pub const COMMAND: packet::Command = packet::Command([0x01, 0x01]);
}

impl ToPacket for RequestState {
    type DirectionMarker = packet::OutboundMarker;

    fn command(&self) -> packet::Command {
        Self::COMMAND
    }

    fn body(&self) -> Vec<u8> {
        Vec::new()
    }
}
```

ANC command verbatim (`common/packet/outbound/set_sound_modes.rs:5-17`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SetSoundModes {
    pub ambient_sound_mode: AmbientSoundMode,
    pub sound_modes: SoundModes,
}

impl SetSoundModes {
    pub const COMMAND: packet::Command = packet::Command([0x06, 0x81]);
```

APK comparison:

- APK ANC `06 81` == `SetSoundModes::COMMAND`. Direct match.
- APK software EQ `02 88` **does not exist anywhere in the repository** (`grep` for `[0x02, 0x88]`, `[2, 136]`, `0x88` as a command finds only unrelated byte values in test vectors). d1202 uses `03 87`; other EQ encoders in `common` use `02 81` (`set_equalizer.rs:3`), `02 83` (`set_equalizer_with_drc.rs:22`), `03 86`/`03 87` (`set_equalizer_and_custom_hear_id.rs:54-56`). A D1204 driver must not assume `03 87`.

---

## 4. D1202 state-update packet body layout

### 4.1 Parser verbatim

`OpenSCQ30/lib/src/devices/soundcore/d1202/packets/inbound/state_update.rs:64-142`:

```rust
impl FromPacketBody for D1202StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "d1202 state update packet",
            map(
                (
                    TwsStatus::take,
                    DualBattery::take,
                    DualFirmwareVersion::take,
                    SerialNumber::take,
                    take(5usize), // case fw? unknown
                    CaseBatteryLevel::take,
                    CommonEqualizerConfiguration::take,
                    take(1usize),
                    CustomHearId::take_with_music_genre_at_end,
                    take(1usize),
                    ButtonStatusCollection::take(
                        d1202::BUTTON_CONFIGURATION_SETTINGS.parse_settings(),
                    ),
                    take(1usize),
                    d1202::structures::SoundModes::take,
                    take(1usize),
                    TouchTone::take,
                    LowBatteryPrompt::take,
                    Ldac::take,
                    take_bool, // dual connections enabled
                    AutoPowerOff::take,
                    LimitHighVolume::take,
                    d1202::structures::SpatialAudio::take,
                ),
                |(
                    tws_status,
                    battery,
                    firmware_version,
                    serial_number,
                    _case_firmware_version,
                    case_battery_level,
                    equalizer_configuration,
                    _unknown1,
                    hear_id,
                    _unknown2,
                    button_configuration,
                    _unknown3,
                    sound_modes,
                    _unknown4,
                    touch_tone,
                    low_battery_prompt,
                    ldac,
                    dual_connections_enabled,
                    auto_power_off,
                    limit_high_volume,
                    spatial_audio,
                )| Self {
                    tws_status,
                    battery,
                    firmware_version,
                    serial_number,
                    case_battery_level,
                    equalizer_configuration,
                    hear_id,
                    button_configuration,
                    sound_modes,
                    touch_tone,
                    low_battery_prompt,
                    ldac,
                    dual_connections_enabled,
                    auto_power_off,
                    limit_high_volume,
                    spatial_audio,
                },
            ),
        )
        .parse_complete(input)
    }
}
```

Serializer verbatim (`state_update.rs:144-180`):

```rust
impl ToPacket for D1202StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        self.tws_status
            .bytes()
            .into_iter()
            .chain(self.battery.bytes())
            .chain(self.firmware_version.bytes())
            .chain(self.serial_number.bytes())
            .chain(std::iter::repeat_n(0, 5))
            .chain(self.case_battery_level.bytes())
            .chain(self.equalizer_configuration.bytes())
            .chain(std::iter::once(0))
            .chain(self.hear_id.bytes_with_music_genre_at_end())
            .chain(std::iter::once(0))
            .chain(
                self.button_configuration
                    .bytes(d1202::BUTTON_CONFIGURATION_SETTINGS.parse_settings()),
            )
            .chain(std::iter::once(0))
            .chain(self.sound_modes.bytes())
            .chain(std::iter::once(0))
            .chain(self.touch_tone.bytes())
            .chain(self.low_battery_prompt.bytes())
            .chain(self.ldac.bytes())
            .chain(std::iter::once(u8::from(self.dual_connections_enabled)))
            .chain(self.auto_power_off.bytes())
            .chain(self.limit_high_volume.bytes())
            .chain(self.spatial_audio.bytes())
            .collect()
    }
}
```

`STATE_COMMAND` is `Command([0x01, 0x01])` (`common/packet/inbound/state.rs:3`).

### 4.2 Byte offsets (relative to packet body, 0-based)

Field sizes derive from: `TwsStatus` 2 (`common/structures/tws_status.rs:45-47`), `DualBattery` 4 (`common/structures/battery.rs:31-38`), `DualFirmwareVersion` 10 (2x5 ASCII, `firmware_version.rs:62-70`), `SerialNumber` 16 (`serial_number.rs:18-23`), EQ 2+2x10=22 (`equalizer_configuration.rs:55-90`), `CustomHearId<2,10>` 48 (`custom_hear_id.rs:84-131`), buttons 8 (`d1202.rs:93-151` + `common/modules/button_configuration.rs:87-90`), `SoundModes` 8 (`d1202/structures.rs:31-46`), `AutoPowerOff` 2 (`auto_power_off.rs:33-35`), `LimitHighVolume` 3 (`limit_high_volume.rs:47-49`), `SpatialAudio` 2 (`d1202/structures.rs:141-143`).

| Body offset | Size | Field | Notes |
|---|---:|---|---|
| 0-1 | 2 | `TwsStatus` | byte 0 = host device (0 left, 1 right), byte 1 = connected bool |
| 2-3 | 2 | battery levels | left, right; display value = raw + 1 (`d1202.rs:75-82`) |
| 4-5 | 2 | charging flags | left, right; 1 == charging |
| 6-10 | 5 | firmware left | ASCII `NN.NN` |
| 11-15 | 5 | firmware right | ASCII `NN.NN` or 0x00/0xFF padding |
| 16-31 | 16 | serial number | ASCII, must be alphanumeric |
| 32-36 | 5 | case firmware (unknown) | parsed as `take(5)`; sample contains `01.83` |
| 37 | 1 | case battery level | raw + 1 for display |
| 38-39 | 2 | EQ preset id | **LE u16** |
| 40-49 | 10 | EQ band gains, left | `packet_byte + MIN_VOLUME` where MIN=-120; 0xFF = absent channel |
| 50-59 | 10 | EQ band gains, right | same |
| 60 | 1 | unknown1 | sample 0 |
| 61 | 1 | hear-ID enabled | bool (==1) |
| 62-71 | 10 | hear-ID EQ left | 0xFF = absent |
| 72-81 | 10 | hear-ID EQ right | 0xFF = absent |
| 82-85 | 4 | hear-ID timestamp | **BE u32** (`custom_hear_id.rs:93`) |
| 86 | 1 | hear-ID type | 0 initial, 1 custom, 2 favorite genre (`hear_id.rs:9-16`) |
| 87-96 | 10 | custom hear-ID EQ left | 0xFF = absent |
| 97-106 | 10 | custom hear-ID EQ right | 0xFF = absent |
| 107-108 | 2 | favorite music genre | **LE u16**, moved to end for d1202 (`take_with_music_genre_at_end`) |
| 109 | 1 | unknown2 | sample 10 |
| 110-117 | 8 | button actions | one byte per button: low nibble = action when TWS connected, high nibble = when disconnected (none = 0xF); order from `d1202.rs:99-108` |
| 118 | 1 | unknown3 | sample 51 (`'3'`) |
| 119 | 1 | ambient sound mode | enum |
| 120 | 1 | manual/adaptive ANC strength packed | `manual_adaptive_noise_canceling_byte` |
| 121 | 1 | transparency mode | 0 full, 1 vocal (`d1202/structures.rs:99-104`) |
| 122 | 1 | noise canceling mode | 0 manual, 1 adaptive, 2 multi-scene (`structures.rs:106-112`) |
| 123 | 1 | wind noise byte | suppression bit + detected bit |
| 124 | 1 | unknown (sound modes) | raw u8 |
| 125 | 1 | multi-scene ANC | 0 transport, 1 outdoor, 2 indoor (`structures.rs:114-120`) |
| 126 | 1 | real-time adaptive ANC | bool |
| 127 | 1 | unknown4 | sample 50 (`'2'`) |
| 128 | 1 | touch tone | bool |
| 129 | 1 | low battery prompt | bool |
| 130 | 1 | LDAC | bool |
| 131 | 1 | dual connections enabled | bool |
| 132 | 1 | auto power off enabled | bool |
| 133 | 1 | auto power off duration index | 2 == 30m in sample |
| 134 | 1 | limit high volume enabled | bool |
| 135 | 1 | dB limit | sample 90 |
| 136 | 1 | dB refresh rate | 0 realtime, 1 10s, 2 1m |
| 137 | 1 | spatial audio enabled | bool |
| 138 | 1 | spatial audio mode | 0 music, 2 movie, 3 gaming (`structures.rs:146-152`) |

Total consumed: **139 bytes**.

SoundModes parser verbatim (`OpenSCQ30/lib/src/devices/soundcore/d1202/structures.rs:55-97`) — this is the 8-byte block at offsets 119-126:

```rust
impl FromPacketBody for SoundModes {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "sound modes",
            map(
                (
                    AmbientSoundMode::take,
                    take_manual_adaptive_noise_canceling,
                    TransparencyMode::take,
                    NoiseCancelingMode::take,
                    WindNoise::take,
                    le_u8,
                    MultiSceneAnc::take,
                    take_bool,
                ),
                |(
                    ambient_sound_mode,
                    (manual_noise_canceling, adaptive_noise_canceling),
                    transparency_mode,
                    noise_canceling_mode,
                    wind_noise,
                    _unknown,
                    multi_scene_anc,
                    real_time_adaptive_anc,
                )| Self {
                    ambient_sound_mode,
                    transparency_mode,
                    noise_canceling_mode,
                    multi_scene_anc,
                    real_time_adaptive_anc,
                    manual_noise_canceling,
                    adaptive_noise_canceling,
                    wind_noise,
                },
            ),
        )
        .parse_complete(input)
    }
}
```

### 4.3 Important test-vector observation

The real captured body used in `OpenSCQ30/lib/src/devices/soundcore/d1202.rs:175-185` is **152 bytes**, while the parser consumes 139. The final 13 bytes (sample: eight `0x00` then five `0xFF`) are silently ignored because `TryToPacket::try_to_packet_raw_error` calls `T::take` and discards the remainder (`common/packet/inbound/inbound_packet.rs:53-57`). Conversely, `D1202StateUpdatePacket::body()` serializes only 139 bytes, so demo packets are shorter than real captures. For D1204, treat bytes 139+ as unmodelled and capture them before assuming the d1202 parser is complete. A five-byte all-`0xFF` tail has the shape of an absent `FirmwareVersion` (`firmware_version.rs:34-40`), suggesting a second/case firmware or companion field.

Sample anchors confirming the table: bytes 0-1 `1,1`; 2-5 `9,9,0,0`; 6-15 `01.8301.83`; 16-31 `12023409C9B403CE`; 38-39 `0,0`; 40-49 `120`*10; 50-59 `255`*10; 61 `0`; 82-85 `0,0,0,0`; 86 `2`; 110-117 `255,255,99,102,255,255,68,68`; 128-138 `1,1,0,1,1,2,0,90,0,0,0`.

---

## 5. EQ configuration

### 5.1 Band/range settings (common module, reused by d1202)

`OpenSCQ30/lib/src/devices/soundcore/common/modules/equalizer.rs:100-104`:

```rust
const COMMON_VISIBLE_BANDS: usize = 8;
const COMMON_PRESET_BANDS: usize = 10;
const COMMON_MIN_VOLUME: i16 = -120;
const COMMON_MAX_VOLUME: i16 = 134;
const COMMON_FRACTION_DIGITS: u8 = 1;
```

`common_settings_type_2()` (`equalizer.rs:261-408`) returns `EqualizerModuleSettings<8, 10, -120, 134, 1>` and is the exact settings object passed by d1202 (`d1202.rs:53-55`, `d1202/modules.rs:16-19`). `common_settings_with_presets` (`equalizer.rs:410-429`) supplies:

```rust
    EqualizerModuleSettings {
        custom_preset_id: Some(0xfefe),
        band_hz: [100, 200, 400, 800, 1600, 3200, 6400, 12800],
        invisible_bands_mode: InvisibleBandsMode::Fixed(vec![0, 0]),
        preset_min_volume: -60,
        preset_max_volume: 60,
        presets,
    }
```

Preset IDs are sequential `0x0000`..`0x0015` (22 presets, `equalizer.rs:263-407`). Raw packet range is -120..134 with 1 fractional digit (so UI -60.0..+60.0 dB); conversion is in `common/structures/volume_adjustments.rs:79-89`:

```rust
    fn signed_adjustment_to_packet_byte(adjustment: i16) -> u8 {
        let clamped = adjustment.clamp(MIN_VOLUME, MAX_VOLUME);
        let shifted = clamped - MIN_VOLUME;
        shifted
            .try_into()
            .expect("value is already clamped, so it can't overflow")
    }

    fn packet_byte_to_signed_adjustment(byte: u8) -> i16 {
        (byte as i16) + MIN_VOLUME
    }
```

`0xFF` means channel/band set absent (`volume_adjustments.rs:46-64`, `OptionalVolumeAdjustmentsExt::bytes` at `:254-258`).

### 5.2 D1202-specific EQ encoder

`d1202/packets/outbound.rs:35-54` documents the 114-byte body layout verbatim:

```rust
    /*
    Example:
    254, 254, // preset id
    0, 0, // hear id favorite music genre
    120, 120, 120, 120, 120, 120, 120, 120, 120, 120, // eq left
    120, 120, 120, 120, 120, 120, 120, 120, 120, 120, // eq right
    0,   // unknown
    0,   // unknown
    1,   // is hear id enabled
    145, 147, 139, 141, 122, 111, 105, 100, 60, 60, // hear id eq left
    145, 147, 139, 141, 122, 111, 105, 100, 60, 60, // hear id eq right
    0, 0, 0, 0, // hear id timestamp
    1, // hear id type
    145, 147, 139, 130, 122, 133, 175, 114, 60, 0, // hear id custom eq left
    145, 147, 139, 130, 122, 133, 175, 114, 60, 0, // hear id custom eq right
    145, 0, 147, 0, 139, 0, 130, 0, 122, 0, 133, 0, 175, 0, 114, 0, 60, 0, 0, 0, // effective eq left with interleaved 0s
    145, 0, 147, 0, 139, 0, 130, 0, 122, 0, 133, 0, 175, 0, 114, 0, 60, 0, 0, 0, // effective eq right with interleaved 0s
    0, // unknown
    0, // unknown
    */
```

The serialisation itself (`d1202/packets/outbound.rs:59-98`) ends with the two interleaved 20-byte blocks and two trailing zero bytes; the source comment at `:56-58` states Hear-ID enablement is intentionally unsupported:

```rust
    // if hear id is favorite music genre, hear id initial should be modified in some unknown way
    // by favorite music genre. Since I don't know how that modification is performed, ignore it
    // for now. We don't support enabling hear id anyway, so it shouldn't matter.
```

### 5.3 DRC and invisible-band quirks

- **DRC is not used by d1202.** The generic DRC matrix lives in `common/structures/volume_adjustments.rs:91-235` (`apply_drc`, 8x8 coefficient table, bands 9/10 reset to `[0, -120]` at `:224-231`) and is applied by `common/packet/outbound/set_equalizer_and_custom_hear_id.rs:105-110` and by `common/modules/equalizer.rs:513-550` (`has_drc: true`). d1202 instead uses its own state modifier (`d1202/modules/equalizer.rs:30-70`) with `set_equalizer_configuration` (`03 87`).
- **Invisible bands**: the shared setting uses `InvisibleBandsMode::Fixed(vec![0, 0])`, but for d1202 band 10 must be `-120`. `d1202/modules/equalizer/state_modifier.rs:177-205` re-implements this when spatial audio is disabled:

```rust
            // Since the equalizer configuration may not have been touched by the common EqualizerSettingHandler,
            // the InvisibleBandsMode may not be in effect. As a workaround, we re-implement how that's handled here.
            // In the future, applying InvisibleBandsMode in the parser would probably be a good idea.
            let equalizer_configuration: &EqualizerConfiguration<_, _, _, _, _> =
                target_state.get();
            let mut volume_adjustments = *equalizer_configuration
                .volume_adjustments_channel_1()
                .copied()
                .unwrap_or_default()
                .adjustments();
            const {
                assert!(
                    BANDS == 10,
                    "We should have enough bands to assign to index 8 and 9",
                );
            }
            volume_adjustments[8] = 0;
            volume_adjustments[9] = MIN_VOLUME;
```

- **Hear-ID state coupling**: `handle_equalizer` forces `target_hear_id.is_enabled = false` and clears `spatial_audio.is_enabled` whenever EQ changes (`d1202/modules/equalizer/state_modifier.rs:127-141`). When enabling spatial audio it sends `10 81`; when disabling it sends another EQ packet instead of `10 81` with `enabled=false` (`:159-205`). This is a device quirk that must be replicated for D1204 only if its app behaves identically.
- **Split ownership**: `common` owns band count/range/presets/settings UI (`common/modules/equalizer.rs`), Hear-ID structs and byte encoding (`common/structures/custom_hear_id.rs`, `hear_id.rs`), and volume math (`volume_adjustments.rs`). `d1202` owns the `03 87` payload, the Hear-ID-at-end variant, and the EQ/spatial state modifier.

---

## 6. Driver structure and minimum changes for `d1204`/`a3956`

### 6.1 Files to copy and adapt

`OpenSCQ30/lib/src/devices/soundcore/d1202.rs` — registry, state fetch, builder wiring, button settings, tests.
`OpenSCQ30/lib/src/devices/soundcore/d1202/state.rs` — `D1202State` + `Update<D1202StateUpdatePacket>`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/structures.rs` — `SoundModes`, `TransparencyMode`, `NoiseCancelingMode`, `MultiSceneAnc`, `SpatialAudio`, `SpatialAudioMode`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/packets.rs`, `packets/outbound.rs`, `packets/inbound.rs`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/packets/inbound/state_update.rs` — `D1202StateUpdatePacket` (`FromPacketBody`, `ToPacket`, `Default`).
`OpenSCQ30/lib/src/devices/soundcore/d1202/modules.rs` — builder extensions `d1202_sound_modes`, `d1202_equalizer`, `d1202_spatial_audio`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/modules/equalizer.rs` + `equalizer/state_modifier.rs`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/modules/sound_modes.rs` + `sound_modes/setting_handler.rs`.
`OpenSCQ30/lib/src/devices/soundcore/d1202/modules/spatial_audio.rs` + `spatial_audio/setting_handler.rs`.

Shared code intentionally reused without copying: `common/device.rs`, `common/macros.rs`, `common/packet*` (framing/transport), `common/settings_manager.rs`, `common/modules/*`, `common/structures/*`.

### 6.2 Traits / structs to rename

- `D1202State` (with `#[derive(Has)]`, `state.rs:19-40`) and `impl Update<D1202StateUpdatePacket>` (`state.rs:74-112`).
- `D1202StateUpdatePacket` with `FromPacketBody<DirectionMarker = InboundMarker>` and `ToPacket<DirectionMarker = InboundMarker>` (`state_update.rs:64,144`).
- `SoundcoreDeviceRegistry<D1202State>: BuildDevice<D1202State>` generated by `soundcore_device!` (`d1202.rs:31`; macro at `common/macros.rs:13-79`).
- `ModuleCollection<D1202State>` extension impls in `modules.rs`, `modules/equalizer.rs`, `modules/sound_modes.rs`, `modules/spatial_audio.rs`.
- `SettingHandler<T>` impls: `SoundModesSettingHandler`, `SpatialAudioSettingHandler` (trait at `common/settings_manager.rs:99-109`).
- `StateModifier<T>` impl: `d1202/modules/equalizer/state_modifier.rs:48-85`; packet handler generated by `state_update_packet_module!` (`common/macros.rs:138-169`).

### 6.3 Minimum change set

1. Register the model: add `SoundcoreD1204` (or `SoundcoreA3956`) to `devices/device_model.rs:32-75` and map it in both `device_registry()` and `demo_device_registry()` (`:134-135`, `:193-194`). Add `pub mod d1204;` to `devices/soundcore.rs:1-38`.
2. UUID: normally **no change** — default dynamic selection already matches `...B395C` (section 1). Only add a `SoundcoreDeviceConfig` with `Constant(uuid!("0cf12d31-fac3-4553-bd80-d6832e7b395c"))` if fixed selection is wanted (pattern at `a3930.rs:95-100`).
3. Framing: **no change** unless a capture confirms the APK's BE length / `09 FF 00 00 00` header; that would require a separate codec, not a d1204-only tweak.
4. State layout: re-derive offsets 0-138 from a D1204 state capture; adjust `state_update.rs:70-97` and `:151-179`. Decide whether to parse the 13 trailing bytes observed in d1202.
5. EQ: replace `03 87` (`d1202/packets/outbound.rs:98`) with APK `02 88` and its payload; verify band count/ranges/preset IDs. Do not reuse the custom Hear-ID interleaving without evidence.
6. Spatial audio: verify `10 81` exists on D1204; if absent, drop `d1202_spatial_audio`, its state field, and the EQ/spatial coupling in `state_modifier.rs`.
7. Buttons/settings: re-capture `BUTTON_CONFIGURATION_SETTINGS` (`d1202.rs:93-151`); 8 buttons/4 press kinds and TWS-low-nibble packing are device-specific.
8. Battery scaling: verify `max_level: 10, level_offset: 1` for earbuds and case (`d1202.rs:75-82`).

---

## 7. Likely D1204 divergences from d1202, based on APK findings

1. **EQ command mismatch is the strongest signal.** APK software EQ is `02 88`; d1202 hardcodes `03 87`. The shared `common` EQ encoders use `02 81`, `02 83`, `03 86`, `03 87` — never `02 88`. Expect a different payload (probably without the 114-byte custom Hear-ID block). Capture before adapting.
2. **Length endianness risk.** APK says `u16-BE`, code uses `le_u16`/`to_le_bytes()` everywhere. If D1204's `D1202CmdService` is a genuine port of D1202C's implementation, LE is more likely and the APK note may describe a different layer; treat as unverified, not as a required change.
3. **Inbound header byte 4.** APK `09 FF 00 00 00 ...` vs code `09 FF 00 00 01 <cmd>`. A raw-socket Rust driver must use the exact bytes observed from D1204.
4. **Dual-core ANC.** D1202's ANC model is a single 8-byte block (`AmbientSoundMode`, packed manual/adaptive strength, transparency, NC mode, wind noise, multi-scene, real-time adaptive). A dual-core device plausibly reports per-core mode/strength; the APK sharing `06 81` only guarantees the command, not the body layout. Re-derive `d1202/structures.rs:18-28,31-46`.
5. **Smart case / Box companion.** d1202 only keeps a 5-byte "case fw? unknown" field (`state_update.rs:78`) plus case battery. 13 unparsed trailing bytes in the d1202 sample (section 4.3, ending in five `0xFF`) look like an absent firmware/serial block. A3956-era devices elsewhere in the tree have dedicated case modules (`a3954/modules/case_language.rs`, `a3954/modules/case_features.rs`, `a3954/modules/easy_chat.rs`) — expect D1204 to add analogous commands/state rather than reusing d1202's reserved bytes.
6. **Extra sensors / features.** d1202 exposes `WindNoise` detection and `RealTimeAdaptiveNoiseCanceling` (`d1202/structures.rs:25-27`). Other A39xx drivers show the shape of likely additions: `a3955/modules/anc_personalized_to_ear_canal.rs`, `a3954/modules/easy_chat.rs`, `a3953/modules/spatial_audio.rs`, `a3953/modules/press_sensitivity.rs`. These are absent from d1202 and should not be assumed present.
7. **Family vs service sharing.** "A3956 family" affects database keys, setting IDs and branding (`DeviceModel` + `CustomEqualizerProfileStore` use `device_model` in `common/modules/equalizer.rs:755-758`), but not the wire protocol. Sharing `D1202CmdService` with D1202C justifies copying the ANC command and possibly the state-request command only; it does **not** justify copying D1202's EQ encoder, spatial module or state offsets wholesale.
8. **Checksum/config.** d1202 uses the default `ChecksumKind::Suffix`; keep it unless a D1204 capture shows no checksum (then set `checksum_kind: ChecksumKind::None` in the macro config, `common/macros.rs:28-34`).
9. **Model identity.** Both D1202 and D1202C already share one driver with no per-model branching (`device_model.rs:134-135`). If D1204 state differs, add a new variant instead of branching inside `d1202.rs`.
10. **Driver-native translation.** For a standalone Rust driver, the minimum viable core is: RFCOMM connect with the masked vendor UUID, `Packet::bytes`/`Packet::take` with LE length and additive checksum, `PacketIOController` buffering/retry, `01 01` state request, the `06 81` ANC setter with the captured body, and only then EQ `02 88` once its payload is captured.

---

## Evidence index (quick reference)

- UUID: `OpenSCQ30/lib/src/devices/soundcore.rs:42-49`; `common/device.rs:88-102`; `api/connection.rs:56-60`; `connection_backend/windows/rfcomm.rs:168-184`.
- Framing: `common/packet.rs:67-109,127-160,186-221`; `common/packet/checksum.rs:1-5`; `common/packet/packet_io_controller.rs:57-138,320-410`; `common/packet/outbound/request_state.rs:28-35`.
- Commands: `d1202/packets/outbound.rs:8-17,98`; `common/packet/outbound/*.rs` (line refs in section 3); `common/modules/sound_modes_v2/state_modifier.rs:44-49`.
- State: `d1202/packets/inbound/state_update.rs:21-39,64-180`; `d1202/structures.rs:18-152`; `d1202.rs:175-185` (152-byte capture); `common/packet/inbound/inbound_packet.rs:53-57`.
- EQ: `common/modules/equalizer.rs:100-104,261-429`; `d1202/packets/outbound.rs:35-98`; `d1202/modules/equalizer/state_modifier.rs:110-209`; `common/structures/volume_adjustments.rs:46-89,91-235`; `common/structures/custom_hear_id.rs:84-147`.
- Driver wiring: `d1202.rs:26-91,93-151`; `common/macros.rs:13-82,138-169`; `devices/device_model.rs:32-75,77-140`.
