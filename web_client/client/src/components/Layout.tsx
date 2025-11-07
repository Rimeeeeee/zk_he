import Navbar from ".//Navbar";

interface Props {
  children: React.ReactNode;
}

export default function AppLayout({ children }: Props) {
  return (
    <div className="min-h-screen bg-linear-to-b from-zinc-900 to-black text-gray-100 relative overflow-hidden">
      <Navbar />
      <div className="pt-20">{children}</div>
    </div>
  );
}
