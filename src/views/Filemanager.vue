<template>
    <div id="container">
        <div id="navigate-bar">
            <div id="back-icon" @click="backToUpper">
                <img src="../assets/backToParent.svg">
            </div>
            <div>
                <el-breadcrumb separator="/" id="file-bc">
                    <el-breadcrumb-item v-for="item in pathBreadCrumb" @click="flashDirContent(item.path)">{{ item.name
                    }}</el-breadcrumb-item>
                </el-breadcrumb>
            </div>
        </div>
        <div id="files-pane" @click.right.self.prevent="paneRightClick">
            <FileItem v-for="file in dirContent" v-bind="file" @delete="onFileDelete" @move="onFileMove"
                @enter="flashDirContent">
            </FileItem>
        </div>
    </div>

    <context-menu v-model:show="showRightClickMenu" :options="FileRightClickOptionsConfig">
        <context-menu-item label="上传视频" @click="handleFilePaneMemu(FileMenuOperate.Upload)" />
        <context-menu-item label="创建文件夹" @click="handleFilePaneMemu(FileMenuOperate.CreateDir)" />
    </context-menu>

    <el-dialog v-model="showCreateDirInput" title="Shipping address">
        <el-input v-model="newDirName" placeholder="Please input name of new directory" />
        <template #footer>
            <span class="dialog-footer">
                <el-button @click="showCreateDirInput = false">Cancel</el-button>
                <el-button type="primary" @click="creatDir">
                    Confirm
                </el-button>
            </span>
        </template>
    </el-dialog>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref } from "vue";
import FileItem from "../components/FileItem.vue";
import { ContextMenu, ContextMenuItem } from '@imengyu/vue3-context-menu';
import { open } from '@tauri-apps/api/dialog';
import { invoke, } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { FileNode } from "../scripts/fs.ts";
import * as fs from "../scripts/fs.ts"

import pathlib from 'path-browserify';
import slash from "slash";

const struct_loaded = ref(false)

onMounted(() => {
    load_structure()
})

const curDir = ref("/")
const pathBreadCrumb = computed(() => {
    const splitPaths = (cur: string,): { path: string, name: string }[] => {
        const path = pathlib.parse(cur)
        const isRoot = path.name === ''

        const item = {
            path: cur,
            name: path.name
        }
        if (isRoot) {
            return [item]
        } else {
            let parent = splitPaths(path.dir)
            parent.push(item)
            return parent
        }
    }

    return splitPaths(curDir.value)
})
const dirContent = ref<FileNode[]>([])

async function load_structure() {
    let tree: FileNode = await invoke("load_dir_tree")
    struct_loaded.value = true
    curDir.value = tree.path

    flashDirContent()
}

async function backToUpper() {
    console.log("back")
    curDir.value = pathlib.dirname(curDir.value)
    flashDirContent()
}

const showRightClickMenu = ref(false)
const FileRightClickOptionsConfig = ref({
    zIndex: 3,
    minWidth: 230,
    x: 500,
    y: 200
})

function paneRightClick(e: MouseEvent) {
    showRightClickMenu.value = true
    FileRightClickOptionsConfig.value.x = e.x
    FileRightClickOptionsConfig.value.y = e.y
}

enum FileMenuOperate {
    Upload,
    CreateDir
}


const showCreateDirInput = ref(false)
const newDirName = ref("")
async function handleFilePaneMemu(opt: FileMenuOperate) {
    switch (opt) {
        case FileMenuOperate.Upload: {
            let paths = await selectFileToUpload()
            for (let p of paths) {
                upload(p)
            }
            break
        }
        case FileMenuOperate.CreateDir: {
            showCreateDirInput.value = true
        }
    }
}

async function selectFileToUpload() {
    const selectedPaths = await open({
        multiple: true,
        directory: false,
    });
    console.log({ selectedPaths })
    if (selectedPaths === null) {
        return []
    }
    if (typeof selectedPaths == "string") {
        return [selectedPaths]
    }
    return selectedPaths
}


class UploadEvent {
    percent!: string;
    is_done: boolean = false
}

function append_file(file: FileNode) {
    dirContent.value.push(file)
    dirContent.value.sort((a, b) => {
        if (a.isDir && b.isDir) {
            return 0
        }
        if (a.isDir) {
            return -1
        }
        if (b.isDir) {
            return 1
        }
        return 0
    })
}

async function upload(path: string) {
    console.log("uploading:", path)

    const toDir = curDir.value
    let event_key: string = await invoke("upload_file", { localPath: path, toDir: toDir })
    const unlisten = await listen<UploadEvent>(event_key, (event) => {
        const filename = pathlib.basename(slash(path))
        const now = new Date().toLocaleString()

        if (event.payload.is_done) {
            unlisten()
            const file = new FileNode(filename, pathlib.join(toDir, filename), now, undefined)
            append_file(file)
            console.log("uploaded", file)
        }
    })
}

async function creatDir() {
    const name = newDirName.value;

    newDirName.value = ""
    showCreateDirInput.value = false

    const path = pathlib.join(curDir.value, name)
    await fs.creatDir(path)

    const now = new Date().toLocaleString()
    const file = new FileNode(name, path, now, [])
    append_file(file)
    console.log("created", { file })
}

async function onFileDelete(path: string) {
    console.log("manager deleting", { path })
    await fs.deleteFile(path)
    dirContent.value = dirContent.value.filter((f) => {
        return f.path !== path
    })
}

async function onFileMove(params: { src: string, receiveDir: string }) {
    await fs.moveFile(params.src, params.receiveDir)
    dirContent.value = dirContent.value.filter((f) => {
        return f.path !== params.src
    })
}

async function flashDirContent(path: string = curDir.value) {
    dirContent.value = await fs.loadDir(path)
    curDir.value = path
}
</script>

<style scoped>
#container {
    display: flex;
    height: 100%;
    flex-direction: column;
}

#navigate-bar {
    display: flex;
    min-height: 2em;
    border-bottom: 1px gray solid;
}

#back-icon {
    width: 2em;
    height: 100%;
    border-right: 1px solid gray;
    padding: 0 10px;
}

#back-icon:hover {
    background: gray;
    cursor: pointer;
}

img {
    width: 100%;
    height: 100%;
}

#file-bc {
    height: 100%;
    font-size: 20px;
    display: flex;
    align-items: center;
}

#files-pane {
    display: flex;
    flex-wrap: wrap;
    padding: .5em .8em;
    flex-grow: 2;
    align-items: start;
    align-content: start;
}
</style>