import { DocTitle, H2, P, UL, Terminal } from "../components/Prose";

const GITHUB = "https://github.com/pamod-madubashana/SoundCore-Desktop";

export default function Troubleshooting() {
  return (
    <article>
      <DocTitle
        eyebrow="docs / troubleshooting"
        title="Troubleshooting"
        lead="If SoundCore-Desktop does not behave as expected, use these steps to diagnose and resolve common issues."
      />

      <H2>The app does not appear in the tray</H2>
      <UL
        items={[
          "Verify that the app is running.",
          "Check operating system tray settings and hidden icon overflow.",
          "On Windows, confirm the app was not blocked by antivirus or security software.",
        ]}
      />

      <H2>Device connection is not detected</H2>
      <UL
        items={[
          "Ensure Bluetooth is enabled on your computer.",
          "Confirm the Soundcore device is paired and connected.",
          "Restart the app after reconnecting the device.",
        ]}
      />

      <H2>Settings are not restored</H2>
      <UL
        items={[
          "Verify the selected profile has been saved in the app.",
          "Reconnect the device and watch for profile restore activity.",
          "If the app cannot communicate with the device, restart both the app and the device.",
        ]}
      />

      <H2>Docs build issues</H2>
      <P>If Docusaurus docs fail to build:</P>
      <Terminal lines={["cd docs", "npm install", "npm run build"]} />
      <P>Then inspect the terminal output for broken links or missing configuration.</P>

      <H2>Build / packaging issues</H2>
      <UL
        items={[
          "Make sure Rust is installed and up to date.",
          "Confirm npm install completed successfully.",
          "If the Tauri build fails, rerun it and review the error messages.",
        ]}
      />
      <Terminal lines={["npm run tauri build"]} />

      <H2>Getting help</H2>
      <P>
        Open an issue on the project repository:{" "}
        <a
          href={`${GITHUB}/issues`}
          className="text-primary underline-offset-4 hover:underline"
        >
          {GITHUB}/issues
        </a>
        . Include your OS, device model, and a short description of the problem.
      </P>
    </article>
  );
}
