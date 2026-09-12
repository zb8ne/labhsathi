import { Route, Routes } from "react-router-dom";
import { HowItWorksScreen } from "./screens/HowItWorksScreen";
import { MainScreen } from "./screens/MainScreen";
import { PrivacyScreen } from "./screens/PrivacyScreen";
import { ResultsScreen } from "./screens/ResultsScreen";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<MainScreen />} />
      <Route path="/results" element={<ResultsScreen />} />
      <Route path="/privacy" element={<PrivacyScreen />} />
      <Route path="/how-it-works" element={<HowItWorksScreen />} />
    </Routes>
  );
}
