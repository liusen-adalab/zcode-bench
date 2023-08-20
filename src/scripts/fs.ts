import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

export interface FileProps {
  name: string;
  path: string;
  lastModified: string;
  children: FileProps[] | undefined;
  isDir: boolean;
}

export class FileNode implements FileProps {
  isDir: boolean;
  constructor(
    public name: string,
    public path: string,
    public lastModified: string,
    public children: FileNode[] | undefined
  ) {
    this.isDir = this.children != undefined;
  }
}

class UploadEvent {
  constructor(
    public percent: string,
    public is_done: boolean = false,
    public path: string,
    public toDir: string
  ) {}
}

import { ref } from "vue";

export async function upload(toDir: string, path: string) {
  console.log("uploading:", path);

  const progress = ref(new UploadEvent("0", false, path, toDir));

  let event_key: string = await invoke("upload_file", {
    local_path: path,
    to_dir: toDir,
  });

  const unlisten = await listen<UploadEvent>(event_key, (event) => {
    progress.value.percent = event.payload.percent;
    progress.value.is_done = event.payload.is_done;
    if (event.payload.is_done) {
      unlisten();
    }
  });
  return progress;
}

export async function deleteFile(path: string) {
  await invoke("delete_file", { path: path });
}

export async function creatDir(path: string) {
  await invoke("create_dir", { path: path });
}

export async function moveFile(from: string, toDir: string) {
  console.log({ from, toDir });
  await invoke("move_to", { from, toDir });
}

export async function loadDir(path: string) {
  let dir: FileNode[] = await invoke("load_dir_content", { path: path });
  dir = dir.map((f) => {
    const lm = new Date(Number(f.lastModified) * 1000).toLocaleString();
    return new FileNode(f.name, f.path, lm, f.children);
  });
  return dir;
}
