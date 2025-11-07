import { Routes, Route } from "react-router-dom";
import AppLayout from "./components/Layout";
import Home from "./pages/Home";
import CreateVote from "./pages/CreateVote";
import GetToken from "./pages/GetToken";

export default function App() {
  return (
    <Routes>
      <Route
        path="/"
        element={
          <AppLayout>
            <Home />
          </AppLayout>
        }
      />
      <Route
        path="/get-token"
        element={
          <AppLayout>
            <GetToken />
          </AppLayout>
        }
      />
      <Route
        path="/create-vote"
        element={
          <AppLayout>
            <CreateVote />
          </AppLayout>
        }
      />
      <Route
        path="/elections"
        element={
          <AppLayout>
            <div className="flex items-center justify-center min-h-[70vh] text-gray-400 text-xl">
              Coming soon: All Elections
            </div>
          </AppLayout>
        }
      />
    </Routes>
  );
}
