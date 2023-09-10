import Chat from "@/components/Chat";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <h1 className="text-xl">OpenAI Proxy JS Demo</h1>
      <Chat />
    </main>
  );
}
