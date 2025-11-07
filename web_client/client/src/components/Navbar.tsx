import { useNavigate, useLocation } from "react-router-dom";

export default function Navbar() {
  const navigate = useNavigate();
  const location = useLocation();

  const NavButton = ({ label, path }: { label: string; path: string }) => {
    const active = location.pathname === path;
    return (
      <button
        onClick={() => navigate(path)}
        className={`px-4 py-2 rounded-lg transition-all duration-200 font-medium ${
          active
            ? "text-cyan-400 bg-zinc-800 shadow-inner shadow-cyan-600/30"
            : "text-gray-300 hover:text-cyan-400 hover:bg-zinc-800/70"
        }`}
      >
        {label}
      </button>
    );
  };

  return (
    <nav className="fixed top-0 left-0 w-full z-50 backdrop-blur-md bg-zinc-900/70 border-b border-zinc-800">
      <div className="max-w-6xl mx-auto px-6 py-3 flex items-center justify-between">
        <h1
          onClick={() => navigate("/")}
          className="text-lg font-semibold cursor-pointer text-cyan-400 hover:text-cyan-300"
        >
          🔐 SecureVotes
        </h1>
        <div className="flex items-center gap-2 sm:gap-4">
          <NavButton label="Home" path="/" />
          <NavButton label="Get Token" path="/get-token" />
          <NavButton label="Create Vote" path="/create-vote" />
          <NavButton label="All Votes" path="/elections" />
        </div>
      </div>
    </nav>
  );
}
