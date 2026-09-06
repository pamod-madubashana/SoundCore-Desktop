import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App.jsx";
import "./index.css";
import { useThemeSetup } from "./theme/ThemeProvider.jsx";

function Boot() {
  const ready = useThemeSetup();
  if (!ready) return null;
  return <App />;
}

createRoot(document.getElementById("root")).render(<Boot />);
