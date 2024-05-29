import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [path, setPath] = useState("");
  const [pythonVersion, setPythonVersion] = useState("");
  const [condaChannel, setCondaChannel] = useState("");
  const [isSetup, setIsSetup] = useState(false);
  const [disabled, setDisabled] = useState(false);

  async function setProject() {
    const path = await invoke("set_project_path");
    setPath(path as string);
  }

  async function checkIsSetUp() {
    const flag = await invoke("is_set_up", { path });
    setIsSetup(flag as boolean);
  }

  useEffect(() => {
    checkIsSetUp();
  }, [path, pythonVersion]);

  async function setup() {
    setDisabled(true);
    await invoke("setup", { path, pythonVersion, condaChannel });
    checkIsSetUp();
    setDisabled(false);
  }

  async function launch_jupyter_lab() {
    await invoke("launch", { path, program: "jupyterlab" });
  }

  async function launch_spyder() {
    await invoke("launch", { path, program: "spyder" });
  }

  if (path === "") {
    return (
      <div className="container">
        <div className="select-project">
          <button onClick={setProject}>Select Project Folder</button>
        </div>
      </div>
    );
  } else {
    return (
      <>
        <div className="container">
          <div>
            Project: {path}
            <button onClick={setProject}>Change Project Folder</button>
          </div>
        </div>
        <div className="container">
          {!isSetup ? (
            <form
              className="row"
              onSubmit={(e) => {
                if (disabled) return;
                e.preventDefault();
                setup();
              }}
            >
              <label>Conda Channel (optional)<input type="text" value={condaChannel} onChange={(e) => setCondaChannel(e.currentTarget.value)} /></label>
              <label>Python Version
              <select onChange={(e) => setPythonVersion(e.currentTarget.value)}>
                <option value=""></option>
                <option value="3.10">3.10</option>
                <option value="3.11">3.11</option>
                <option value="3.12">3.12</option>
              </select>
              </label>
              {disabled ? (
                <p>Setting up...</p>
              ) : (
                <button type="submit" disabled={disabled}>
                  Set up project
                </button>
              )}
            </form>
          ) : (
            <>
              <button onClick={launch_spyder}>Launch Spyder</button>
              <button onClick={launch_jupyter_lab}>Launch Jupyter Lab</button>
            </>
          )}
        </div>
      </>
    );
  }
}

export default App;
