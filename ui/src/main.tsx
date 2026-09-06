import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./index.css";
import { useThemeSetup } from "./theme/ThemeProvider";

function Boot(): React.JSX.Element {
  const ready = useThemeSetup();
  if (!ready) return null as unknown as React.JSX.Element;
  return <App />;
}

createRoot(document.getElementById("root")!).render(<Boot />);
