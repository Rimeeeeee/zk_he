import { useNavigate, useLocation } from "react-router-dom";
import { motion } from "framer-motion";

export default function Home() {
  const navigate = useNavigate();
  const location = useLocation();

  const NavButton = ({
    label,
    path,
  }: {
    label: string;
    path: string;
  }) => {
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
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 flex flex-col items-center justify-center relative overflow-hidden">
      <div className="absolute inset-0">
        <div className="absolute top-32 left-1/4 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl" />
        <div className="absolute bottom-32 right-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl" />
      </div>
      {/* Main Hero Section */}
      <motion.div
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8 }}
        className="z-10 flex flex-col items-center text-center space-y-6 px-6 pt-24"
      >
        <h1 className="text-5xl md:text-6xl p-3 font-bold bg-clip-text text-transparent bg-linear-to-r from-cyan-400 via-blue-400 to-purple-500 drop-shadow-md">
          🔐 Private Voting System
        </h1>

        <p className="text-gray-400 max-w-2xl text-lg">
          Cast your vote with complete privacy. All votes are{" "}
          <span className="text-cyan-400">encrypted</span> end-to-end using
          advanced homomorphic encryption.
        </p>

        <div className="flex flex-col sm:flex-row gap-4 pt-6">
          <button
            onClick={() => navigate("/get-token")}
            className="px-8 py-3 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-semibold shadow-lg shadow-cyan-500/30 transition-transform hover:scale-105"
          >
            🎟️ Get Token
          </button>
          <button
            onClick={() => navigate("/election/1")}
            className="px-8 py-3 rounded-xl bg-purple-700 hover:bg-purple-600 text-white font-semibold shadow-lg shadow-purple-500/30 transition-transform hover:scale-105"
          >
            🗳️ Go Vote
          </button>
        </div>

        <div className="pt-12 text-gray-500 text-sm">
          © {new Date().getFullYear()} SecureVotes — Powered by TFHE
        </div>
      </motion.div>
    </div>
  );
}
