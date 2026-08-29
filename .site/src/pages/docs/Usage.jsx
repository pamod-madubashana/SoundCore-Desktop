import { DocTitle, H2, P, UL, OL, Note } from "../components/Prose";

export default function Usage() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / usage"
        title="Usage"
        lead="How to use SoundCore-Desktop once it is installed."
      />

      <H2>Launch the app</H2>
      <P>
        After installation, launch SoundCore-Desktop. The app will appear in your system tray.
      </P>

      <H2>Access controls</H2>
      <UL
        items={[
          "Left-click the tray icon to open the control panel.",
          "Right-click the tray icon for the context menu.",
        ]}
      />

      <H2>Configure your Soundcore device</H2>
      <OL
        items={[
          "Connect your Soundcore device to your computer.",
          "Open the SoundCore-Desktop tray window.",
          "Select the saved profile settings you want to apply.",
          'Use the "Apply" or "Save" controls to persist the configuration.',
        ]}
      />

      <H2>Restore settings automatically</H2>
      <P>
        Once a profile is configured, SoundCore-Desktop will detect the device connection event and
        restore your selected Soundcore settings automatically.
      </P>

      <H2>Typical workflow</H2>
      <OL
        items={[
          "Start SoundCore-Desktop.",
          "Pair or connect your Soundcore device.",
          "Configure the preferred mode, ANC state, and equalizer profile.",
          "Save the profile configuration.",
          "Reconnect the device later; the app will reapply the saved profile.",
        ]}
      />

      <Note>
        The app is designed to work without the official Soundcore mobile app once your profile is
        configured. If the device is not detected, check Bluetooth connectivity and app permissions.
      </Note>
    </article>
  );
}
