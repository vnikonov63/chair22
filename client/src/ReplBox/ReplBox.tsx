import { useState } from "react";

export default function ReplBox() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    const res = await fetch("http://127.0.0.1:8080/eval", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ text: input }),
    });

    const data = await res.json();
    setOutput(data.result);
  }

  return (
    <div style={{ fontFamily: "monospace" }}>
      <form onSubmit={handleSubmit}>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Enter expression"
        />
        <button type="submit">Run</button>
      </form>
      <pre>{output}</pre>
    </div>
  );
}
