import { useState } from "react";
import { motion } from "framer-motion";
import { useNavigate } from "react-router-dom";
import axios from "axios";

interface Candidate {
  id: number;
  name: string;
}

export default function CreateVote() {
  const navigate = useNavigate();
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [startTime, setStartTime] = useState("");
  const [endTime, setEndTime] = useState("");
  const [candidates, setCandidates] = useState<Candidate[]>([
    { id: 1, name: "" },
  ]);
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState("");

  const addCandidate = () => {
    setCandidates((prev) => [...prev, { id: prev.length + 1, name: "" }]);
  };

  const removeCandidate = (id: number) => {
    setCandidates((prev) => prev.filter((c) => c.id !== id));
  };

  const updateCandidate = (id: number, name: string) => {
    setCandidates((prev) =>
      prev.map((c) => (c.id === id ? { ...c, name } : c))
    );
  };

  const handleCreate = async () => {
    if (!title.trim() || candidates.some((c) => !c.name.trim())) {
      alert("Please fill in all required fields");
      return;
    }

    if (!endTime) {
      alert("Please set an end time for the election");
      return;
    }

    const startTimestamp = startTime
      ? Math.floor(new Date(startTime).getTime() / 1000)
      : Math.floor(Date.now() / 1000);
    const endTimestamp = Math.floor(new Date(endTime).getTime() / 1000);

    if (endTimestamp <= startTimestamp) {
      alert("End time must be after the start time.");
      return;
    }

    setLoading(true);
    setSuccess(false);
    setError("");

    try {
      const res = await axios.post("http://localhost:8080/admin/elections", {
        name: title,
        start_time: startTimestamp,
        end_time: endTimestamp,
        candidates: candidates.map((c, i) => ({ id: i + 1, name: c.name })),
      });

      const id = res.data.election_id;
      setSuccess(true);

      setTimeout(() => navigate(`/election/${id}`), 1500);
    } catch (err) {
      console.error(err);
      setError("Failed to create election. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 flex flex-col items-center justify-center relative overflow-hidden">
      <div className="absolute inset-0">
        <div className="absolute top-32 left-1/4 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl" />
        <div className="absolute bottom-32 right-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl" />
      </div>

      <motion.div
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8 }}
        className="z-10 w-full max-w-2xl bg-zinc-900/80 border border-zinc-800 rounded-2xl shadow-lg p-8 backdrop-blur-md"
      >
        <h1 className="text-3xl font-bold text-center mb-6 bg-clip-text text-transparent bg-linear-to-r from-cyan-400 to-purple-500">
          🗳️ Create New Vote
        </h1>

        <div className="space-y-4">
          <div>
            <label className="block text-gray-400 mb-1 text-sm">
              Election Title
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Enter election name"
              className="w-full bg-zinc-800 text-gray-100 px-4 py-2 rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-cyan-500"
            />
          </div>

          <div>
            <label className="block text-gray-400 mb-1 text-sm">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Optional description"
              className="w-full bg-zinc-800 text-gray-100 px-4 py-2 rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-cyan-500 h-24 resize-none"
            />
          </div>

          <div className="grid sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-gray-400 mb-1 text-sm">
                Start Time
              </label>
              <input
                type="datetime-local"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                className="w-full bg-zinc-800 text-gray-100 px-4 py-2 rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-cyan-500"
              />
            </div>

            <div>
              <label className="block text-gray-400 mb-1 text-sm">
                End Time
              </label>
              <input
                type="datetime-local"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                className="w-full bg-zinc-800 text-gray-100 px-4 py-2 rounded-lg border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-cyan-500"
              />
            </div>
          </div>

          <div>
            <label className="block text-gray-400 mb-2 text-sm">
              Candidates
            </label>
            <div className="space-y-3">
              {candidates.map((c) => (
                <div
                  key={c.id}
                  className="flex items-center gap-2 bg-zinc-800 rounded-lg px-4 py-2 border border-zinc-700"
                >
                  <input
                    type="text"
                    value={c.name}
                    onChange={(e) => updateCandidate(c.id, e.target.value)}
                    placeholder={`Candidate ${c.id}`}
                    className="flex-1 bg-transparent outline-none text-gray-100 placeholder-gray-500"
                  />
                  {candidates.length > 1 && (
                    <button
                      onClick={() => removeCandidate(c.id)}
                      className="text-red-400 hover:text-red-500 transition"
                    >
                      ✕
                    </button>
                  )}
                </div>
              ))}
            </div>

            <button
              onClick={addCandidate}
              className="mt-3 px-4 py-2 rounded-lg bg-cyan-700 hover:bg-cyan-600 text-white font-medium shadow-md transition-transform hover:scale-105"
            >
              + Add Candidate
            </button>
          </div>

          <div className="pt-6">
            <button
              onClick={handleCreate}
              disabled={loading}
              className="w-full py-3 rounded-xl bg-purple-700 hover:bg-purple-600 text-white font-semibold shadow-lg shadow-purple-500/30 transition-transform hover:scale-105"
            >
              {loading ? "Creating..." : "Create Vote"}
            </button>
          </div>

          {success && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="text-green-400 text-center font-medium pt-4"
            >
              ✅ Vote created successfully! Redirecting...
            </motion.div>
          )}

          {error && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              className="text-red-400 text-center font-medium pt-4"
            >
              {error}
            </motion.div>
          )}
        </div>
      </motion.div>
    </div>
  );
}
