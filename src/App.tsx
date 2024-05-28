import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [projectPath, setProjectPath] = useState("");
  const [pythonVersion, setPythonVersion] = useState("");
  const [response, setResponse] = useState("");

  async function setup() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setResponse(await invoke("setup", { projectPath, pythonVersion }));
  }

  async function launch_jupyter_lab() {
    setResponse(await invoke("launch", { projectPath, program: "jupyterlab" }));
  }

  async function launch_spyder() {
    setResponse(await invoke("launch", { projectPath, program: "spyder" }));
  }

  return (
    <>
    <div className="container">
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          setup();
        }}
      >
        <input
          id="project-path-input"
          onChange={(e) => setProjectPath(e.currentTarget.value)}
          placeholder="Enter the project path..."
        />
        <input
          id="python-version-input"
          onChange={(e) => setPythonVersion(e.currentTarget.value)}
          placeholder="Enter the python version..."
        />
        <button type="submit">Set up project</button>
      </form>
    </div>

    <button onClick={launch_spyder}>Launch Spyder</button>
    <button onClick={launch_jupyter_lab}>Launch Jupyter Lab</button>

    <p>{response}</p>
    </>
  );
}

export default App;
