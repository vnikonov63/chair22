import ReplBox from "./ReplBox/ReplBox";
import { useEffect, useState } from "react";

function App() {
  const [replId, setReplId] = useState<number | null>(null);

  useEffect(() => {
    const existing = localStorage.getItem("replId");
    if (existing) {
      setReplId(Number(existing));
      return;
    }

    const API = import.meta.env.VITE_API_BASE;
    fetch(`${API}/repl`, { method: "POST" })
      .then((r) => r.json())
      .then((d) => {
        if (typeof d?.id === "number") {
          localStorage.setItem("replId", String(d.id));
          setReplId(d.id);
        }
      });
  }, []);

  return <ReplBox replId={replId} />;
}

export default App;
