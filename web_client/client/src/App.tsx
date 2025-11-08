import { Routes, Route } from "react-router-dom";
import AppLayout from "./components/Layout";
import Home from "./pages/Home";
import CreateVote from "./pages/CreateVote";
import GetToken from "./pages/GetToken";
import AllVotes from "./pages/AllVotes";
import ElectionDetail from "./pages/ElectionDetails";

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
            <AllVotes />
          </AppLayout>
        }
      />
      <Route
        path="/election/:id"
        element={
          <AppLayout>
            <ElectionDetail />
          </AppLayout>
        }
      />
    </Routes>
  );
}
