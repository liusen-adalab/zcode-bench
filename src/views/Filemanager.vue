<template>
    <div id="files-pane" @click.right.self.prevent="paneRightClick">
        <div class="fileItem" v-for="file in files">
            <FileItem :name="file.name" :isDir="file.isDir" :last-modified="file.lastModified">
            </FileItem>
        </div>
    </div>
    <context-menu v-model:show="showRightClickMenu" :options="FileRightClickOptionsConfig">
        <context-menu-item label="上传视频" @click="onMenuItemClick(FileMenuOperate.Upload)" />
    </context-menu>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import FileItem from "../components/FileItem.vue";
import { ContextMenu, ContextMenuItem } from '@imengyu/vue3-context-menu';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";

const files = ref([
    {
        name: "dir-1",
        isDir: true,
        lastModified: "2020-08-03"
    },
    {
        name: "file-1",
        isDir: false,
        lastModified: "2020-08-03"
    },
])

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
    Upload
}

async function onMenuItemClick(opt: FileMenuOperate) {
    switch (opt) {
        case FileMenuOperate.Upload: {
            let paths = await selectFile()
            await upload(paths)
        }
    }
}


async function selectFile() {
    const selectedPaths = await open({
        multiple: true,
        directory: false,
        // filters: [{
        //     name: 'Video',
        //     extensions: ['mp4', 'mkv', 'webm']
        // }]
    });
    console.log("paths", selectedPaths)
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
}

async function upload(paths: string[]) {
    console.log("uploading:", paths)
    if (paths.length === 0) {
        return
    }
    const remotePath = "C:/Users/OP 005/workspace/test-upload.txt"
    let taskId: number = await invoke("upload_file", { localPath: paths[0], remotePath: remotePath })
    const unlisten = await listen<UploadEvent>(`slice-uploaded-${taskId}`, (event) => {
        console.log(event)
        if (Number(event.payload.percent) >= 100) {
            unlisten()
        }
    })
}


</script>

<style>
.fileItem {
    width: 150px;
    height: 120px;
    /* flex: initial; */
}

#files-pane {
    height: 70lvh;
    display: flex;
    margin: 1em;
    resize: horizontal;
}
</style>