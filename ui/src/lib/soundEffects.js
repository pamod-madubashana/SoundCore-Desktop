// Shared derivation for the "Sound Effects" radio group.
//
// The three modes (Spatial Audio / Default preset / Custom EQ) are mutually
// exclusive *in firmware*: writing any equalizer value clears spatial audio
// (see OpenSCQ30 devices/soundcore/d1202/modules/equalizer/state_modifier.rs).
// So the UI must never keep its own copy of "which mode is on" — it derives it
// from the live snapshot instead, which makes it impossible to drift.
//
// Snapshot shapes that matter here (serde, camelCase, `type` tag):
//   spatialAudio           -> { type: "toggle", value: boolean }
//   spatialAudioMode       -> { type: "select", setting: { options, localizedOptions }, value }
//   presetEqualizerProfile -> { type: "presetEqualizerProfileSelect",
//                               equalizer, select: { options, localizedOptions },
//                               presets, value: string | null }
//   volumeAdjustments      -> { type: "equalizer", setting: { bandHz, fractionDigits, min, max },
//                               readOnly: boolean, value: number[] }
//
// Note `presetEqualizerProfile` nests its options under `select`, NOT `setting`.

export const SECTION_SPATIAL = "spatial";
export const SECTION_DEFAULT = "default";
export const SECTION_CUSTOM = "custom";

/** True when the device reports spatial audio as enabled. `value` is a real boolean. */
export function isSpatialOn(s) {
  return s?.spatialAudio?.value === true;
}

/** The active built-in preset id, or null when a custom curve is loaded. */
export function devicePresetId(s) {
  return s?.presetEqualizerProfile?.value ?? null;
}

export function hasSpatial(s) {
  return !!s?.spatialAudioMode;
}
export function hasDefault(s) {
  return !!s?.presetEqualizerProfile;
}
export function hasCustom(s) {
  return !!s?.volumeAdjustments;
}

/**
 * Which card should read as selected. Order matters: spatial audio wins because
 * the device keeps its last preset id around while spatial audio is engaged.
 */
export function deriveActiveSection(s) {
  if (hasSpatial(s) && isSpatialOn(s)) return SECTION_SPATIAL;
  if (hasDefault(s) && devicePresetId(s) != null) return SECTION_DEFAULT;
  if (hasCustom(s)) return SECTION_CUSTOM;
  return "";
}

/**
 * Built-in presets as `[{ id, label }]`. `id` is what the device expects,
 * `label` is the localized text the backend already ships alongside it.
 */
export function presetEntries(presetSetting) {
  const select = presetSetting?.select;
  const options = select?.options ?? [];
  const localized = select?.localizedOptions ?? [];
  return options.map((id, i) => ({ id, label: localized[i] || prettyPresetId(id) }));
}

/** Same as `presetEntries`, keyed off a whole settings map. */
export function presetOptions(s) {
  return presetEntries(s?.presetEqualizerProfile);
}

/** Fallback for when localizedOptions is missing: "BassBooster" -> "Bass Booster". */
export function prettyPresetId(id) {
  return String(id).replace(/([a-z0-9])([A-Z])/g, "$1 $2");
}

/** Localized label for a built-in preset id, falling back to a prettified id. */
export function presetLabel(s, id) {
  if (id == null) return null;
  return presetOptions(s).find((o) => o.id === id)?.label ?? prettyPresetId(id);
}

/** Localized label for the current spatial audio mode (Music / Movie / Gaming). */
export function spatialModeLabel(s) {
  const setting = s?.spatialAudioMode;
  if (!setting) return null;
  const i = (setting.setting?.options ?? []).indexOf(setting.value);
  return (i >= 0 ? setting.setting?.localizedOptions?.[i] : null) || setting.value || null;
}

const bandsKey = (bands) => (Array.isArray(bands) ? bands.join(",") : "");

/** Name of the saved custom preset whose bands match exactly, or null. */
export function matchCustomPreset(bands, eqPresets = []) {
  const key = bandsKey(bands);
  if (!key) return null;
  return eqPresets.find((p) => p.bands === key)?.name ?? null;
}

/** Summary text for the "Sound Effects" row and the Custom EQ card. */
export function soundEffectLabel(s, eqPresets = []) {
  switch (deriveActiveSection(s)) {
    case SECTION_SPATIAL: {
      const mode = spatialModeLabel(s);
      return mode ? `Spatial Audio · ${mode}` : "Spatial Audio";
    }
    case SECTION_DEFAULT:
      return presetLabel(s, devicePresetId(s)) ?? "Default";
    case SECTION_CUSTOM:
      return matchCustomPreset(s?.volumeAdjustments?.value, eqPresets) ?? "Custom";
    default:
      return "";
  }
}
