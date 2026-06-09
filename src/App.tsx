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
  const [availableSheets, setAvailableSheets] = useState<string[]>([]);
  const [loadingSheets, setLoadingSheets] = useState(false);
  const [status, setStatus] = useState<{ type: "success" | "error"; message: string } | null>(null);
  const [processing, setProcessing] = useState(false);

  async function browseSource() {
    const path = await open({
      multiple: false,
      filters: [{ name: "Spreadsheet Files", extensions: ["xlsx", "xls", "csv"] }],
    });
    if (typeof path === "string") {
      setSourceFile(path);
      setOutputFile(path.replace(/\.\w+$/, "_output.xlsx"));
      setSheetName("");
      setAvailableSheets([]);
      setLoadingSheets(true);
      try {
        const sheets = await invoke<string[]>("get_sheet_names", { sourcePath: path });
        setAvailableSheets(sheets);
        if (sheets.length === 1) setSheetName(sheets[0]);
      } catch (e) {
        setStatus({ type: "error", message: `Failed to read sheets: ${e}` });
      } finally {
        setLoadingSheets(false);
      }
    }
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
        <select
          value={sheetName}
          onChange={(e) => setSheetName(e.target.value)}
          disabled={loadingSheets || availableSheets.length === 0}
        >
          {loadingSheets ? (
            <option value="">Loading sheets…</option>
          ) : availableSheets.length === 0 ? (
            <option value="">Select a source file first</option>
          ) : (
            <>
              {availableSheets.length > 1 && <option value="">— select a sheet —</option>}
              {availableSheets.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </>
          )}
        </select>
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
          "Process File"
        )}
      </button>
    </main>
  );
}

export default App;