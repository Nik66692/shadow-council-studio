import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles.css";
// Load density overrides after the base system so desktop and 4K layouts stay compact.
import "./density.css";
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
