import { invoke } from "@tauri-apps/api/tauri";
import { open, save } from "@tauri-apps/api/dialog";
import { readTextFile, writeFile } from "@tauri-apps/api/fs";

let editor;

function initMonaco() {
  require.config({ paths: { vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.52.2/min/vs" } });
  require(["vs/editor/editor.main"], () => {
    monaco.languages.register({ id: "talea" });
    monaco.languages.setMonarchTokensProvider("talea", {
      tokenizer: {
        root: [
          [/\b(let|print|count|tokenize|doc2bow|split|use)\b/, "keyword"],
          [/#[^$]*/, "comment"]
        ]
      }
    });
    editor = monaco.editor.create(document.getElementById("editor"), {
      value: "",
      language: "talea",
      theme: "vs-dark",
    });
  });
}

export async function runCode() {
  const code = editor.getValue();
  try {
    const output = await invoke("run_talea", { code });
    appendConsole(output);
  } catch (e) {
    appendConsole(String(e));
  }
}

export async function openFile() {
  const selected = await open({ filters: [{ name: "Talea", extensions: ["talea"] }] });
  if (selected) {
    const contents = await readTextFile(selected);
    editor.setValue(contents);
  }
}

export async function saveFile() {
  const filePath = await save({ filters: [{ name: "Talea", extensions: ["talea"] }] });
  if (filePath) {
    await writeFile({ path: filePath, contents: editor.getValue() });
  }
}

function appendConsole(text) {
  const el = document.getElementById("console");
  el.textContent += text + "\n";
  el.scrollTop = el.scrollHeight;
}

window.runCode = runCode;
window.openFile = openFile;
window.saveFile = saveFile;

initMonaco();
