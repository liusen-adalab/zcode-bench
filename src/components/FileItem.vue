<template>
    <div class="file-container" draggable="true" @dragstart="dragStart" :class="folderHoverClass" @dragover="fileHover"
        @dragleave="dragleave" @drop="dragDrop" @dblclick="onDblClick" @click.right.prevent.stop="onRightClick">
        <div class="file-icon">
            <img :src="icon" class="file-svg" />
        </div>
        <div class="file-name">
            <span>
                {{ props.name }}
            </span>
        </div>
    </div>
    <context-menu v-model:show="showRightClickMenu" :options="FileRightClickOptionsConfig">
        <context-menu-item label="delete" @click="onMenuItemClick(FileMenuOperate.Delete)" />
    </context-menu>
</template>

<script lang="ts" setup>
import folder from "../assets/folder.svg"
import file from "../assets/file.svg"
import { ContextMenu, } from '@imengyu/vue3-context-menu';
import { computed, ref } from "vue";
import { FileProps } from "../scripts/fs";

const props = defineProps<FileProps>()
const emit = defineEmits(['delete', 'move', 'enter'])

const icon = computed(() => {
    return props.isDir ? folder : file
})

const folderHoverClass = ref({
    dragHover: false
})

function dragStart(event: DragEvent) {
    event.dataTransfer?.setData("name", props.name)
    event.dataTransfer?.setData("path", props.path)
}

function fileHover(event: DragEvent) {
    event.preventDefault()

    folderHoverClass.value.dragHover = true
}

function dragleave(_event: Event) {
    folderHoverClass.value.dragHover = false
}

function dragDrop(event: DragEvent) {
    const path = event.dataTransfer?.getData('path');
    folderHoverClass.value.dragHover = false

    if (props.isDir) {
        console.log(`moving file or dir '${path}' into ${props.name}`)
        emit("move", { src: path, receiveDir: props.path })
    } else {
        console.log("cannot drop into file")
    }
}

const FileRightClickOptionsConfig = ref({
    zIndex: 3,
    minWidth: 230,
    x: 500,
    y: 200
})

function onRightClick(e: MouseEvent) {
    console.log("item right clicked", e)

    showRightClickMenu.value = true
    FileRightClickOptionsConfig.value.x = e.x
    FileRightClickOptionsConfig.value.y = e.y
}

enum FileMenuOperate {
    Delete
}

const showRightClickMenu = ref(false)

function onMenuItemClick(operate: FileMenuOperate) {
    switch (operate) {
        case FileMenuOperate.Delete: {
            console.log(props)
            console.log("deleting file:", props.name)
            emit("delete", props.path)
        }
    }
}


function onDblClick() {
    emit('enter', props.path)
}
</script>


<style>
.file-container {
    width: 7em;
    display: flex;
    flex-direction: column;
    align-items: center;
}

.file-container:hover {
    background-color: rgb(80, 79, 77);
    cursor: pointer;
}

.dragHover {
    background-color: rgb(80, 79, 77);
}

.file-icon {
    width: 100%;
    height: 6em;
    display: flex;
    justify-content: center;
    align-items: center;
}

.file-svg {
    max-width: 100%;
    max-height: 6em;
}

.file-name {
    width: 70%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: center;
}
</style>