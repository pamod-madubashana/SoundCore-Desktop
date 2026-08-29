import { Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import DocsLayout from "./pages/DocsLayout";
import Intro from "./pages/docs/Intro";
import Installation from "./pages/docs/Installation";
import Features from "./pages/docs/Features";
import Usage from "./pages/docs/Usage";
import Architecture from "./pages/docs/Architecture";
import Development from "./pages/docs/Development";
import Deployment from "./pages/docs/Deployment";
import Troubleshooting from "./pages/docs/Troubleshooting";
import Downloads from "./pages/Downloads";
import NotFound from "./pages/NotFound";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/docs" element={<DocsLayout />}>
        <Route index element={<Intro />} />
        <Route path="installation" element={<Installation />} />
        <Route path="features" element={<Features />} />
        <Route path="usage" element={<Usage />} />
        <Route path="architecture" element={<Architecture />} />
        <Route path="development" element={<Development />} />
        <Route path="deployment" element={<Deployment />} />
        <Route path="troubleshooting" element={<Troubleshooting />} />
      </Route>
      <Route path="/downloads" element={<Downloads />} />
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}
