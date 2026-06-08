import { useState } from "react";
import { flushSync } from "react-dom";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import "./App.css";

function App() {
  const [sourceFile, setSourceFile] = useState("");
  const [outputFile, setOutputFile] = useState("");
  const [sheetName, setSheetName] = useState("");
  const [searchStr, setSearchStr] = useState("");
  const [status, setStatus] = useState<{ type: "success" | "error"; message: string } | null>(null);
  const [processing, setProcessing] = useState(false);

  async function browseSource() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Excel Files", extensions: ["xlsx", "xls"] }],
    });
    if (typeof path === "string") setSourceFile(path);
  }

  async function browseOutput() {
    const path = await save({
      defaultPath: "output.xlsx",
      filters: [{ name: "Excel Files", extensions: ["xlsx"] }],
    });
    if (typeof path === "string") setOutputFile(path);
  }

  async function handleProcess() {
    if (!sourceFile || !outputFile || !sheetName || !searchStr) {
      setStatus({ type: "error", message: "Please fill in all fields." });
      return;
    }
    flushSync(() => {
      setProcessing(true);
      setStatus(null);
    });
    try {
      const result = await invoke<string>("process_excel", {
        sourcePath: sourceFile,
        outputPath: outputFile,
        sheetName,
        searchStr,
      });
      setStatus({ type: "success", message: result });
    } catch (e) {
      setStatus({ type: "error", message: String(e) });
    } finally {
      setProcessing(false);
    }
  }

  return (
    <main className="container">
      <div className="field">
        <label>Source File:</label>
        <input type="text" value={sourceFile} readOnly />
        <button onClick={browseSource}>Browse</button>
      </div>

      <div className="field">
        <label>Output File:</label>
        <input type="text" value={outputFile} readOnly />
        <button onClick={browseOutput}>Browse</button>
      </div>

      <div className="field">
        <label>Sheet Name:</label>
        <input
          type="text"
          value={sheetName}
          onChange={(e) => setSheetName(e.target.value)}
        />
      </div>

      <div className="field">
        <label>Search String (e.g., 'Jan/25'):</label>
        <input
          type="text"
          value={searchStr}
          onChange={(e) => setSearchStr(e.target.value)}
        />
      </div>

      {status && <div className={`status ${status.type}`}>{status.message}</div>}

      <button className="process-btn" onClick={handleProcess} disabled={processing}>
        {processing ? (
          <>
            <span className="spinner" />
            Processing...
          </>
        ) : (
          "Process Excel File"
        )}
      </button>
    </main>
  );
}

export default App;