<template>
    <div class="folderContainer" draggable="true" :class="folderHoverClass" @dragstart="dragStart" @dragover="fileHover"
        @drop="dragDrop" @dragleave="dragleave">
        <div class="folderWrapper" @click.right.prevent.stop="rightClick">
            <div class="folder" v-if="props.is_dir">
                <div class="front"></div>
                <div class="center"></div>
                <div class="back"></div>
            </div>
            <div v-else class="file-icon">
                <div>
                    <VideoPlay style="width: 5em; height: 5em; margin-right: 0px; padding-left: 10px; margin-top: -10px;" />
                    <!-- <el-icon :size="70">
                    </el-icon> -->
                </div>
            </div>

            <div class="folderName">
                <span>{{ props.name }}</span>
            </div>
            <div class="folderTime">
                <span>{{ props.last_modified }}</span>
            </div>
        </div>
    </div>
    <context-menu v-model:show="show" :options="FileRightClickOptionsConfig">
        <context-menu-item label="delete" @click="onMenuItemClick(FileMenuOperate.Delete)" />
    </context-menu>
</template>

<script lang="ts" setup>
import { ContextMenu, } from '@imengyu/vue3-context-menu';
import { ref } from 'vue';

enum FileMenuOperate {
    Delete
}

const show = ref(false)

function onMenuItemClick(operate: FileMenuOperate) {
    switch (operate) {
        case FileMenuOperate.Delete: {
            console.log("deleting file:", props.name)
        }
    }
}

const FileRightClickOptionsConfig = ref({
    zIndex: 3,
    minWidth: 230,
    x: 500,
    y: 200
})

export interface FileProps {
    name: string
    is_dir: boolean
    last_modified: string
}

const props = withDefaults(defineProps<FileProps>(), { name: "unknown", is_dir: false, last_modified: "2021/07/17 10:49" })


const folderHoverClass = ref({
    fileshover: false
})

function dragStart(event: DragEvent) {
    console.log("start", event)
    event.dataTransfer?.setData("name", props.name)
}

function fileHover(event: DragEvent) {
    console.log("hover", event)
    event.preventDefault()

    folderHoverClass.value.fileshover = true
}

function dragDrop(event: DragEvent) {
    const data = event.dataTransfer?.getData('name'); // 获取文件的数据
    folderHoverClass.value.fileshover = false
    if (props.is_dir) {
        console.log(`moving file or dir '${data}' into ${props.name}`)
    } else {
        console.log("cannot drop into file")
    }
}

function dragleave(event: Event) {
    console.log("leave ", event)
    folderHoverClass.value.fileshover = false
}

function rightClick(e: MouseEvent) {
    console.log(e)
    console.log("prevented")

    show.value = true
    FileRightClickOptionsConfig.value.x = e.x
    FileRightClickOptionsConfig.value.y = e.y
}
</script>

<style scoped>
.folderContainer {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;

    flex: 0 1 auto
}

.folder {
    width: 100px;
    height: 80px;
    perspective: 600px;
    transform-style: preserve-3d;
}

.folderWrapper {
    padding: 20px 20px 10px 20px;
    position: relative;
    transition: all .2s ease;
    border-radius: 6px;
    cursor: pointer;
}

.folderWrapper:hover {
    background-color: #6c6363;
}

.folder div.back,
.folder div.front {
    position: absolute;
    top: 0;
    left: 5px;
    width: 90px;
    height: 70px;
    background-color: #c5b7f1;
}

.folder div.center {
    width: 80px;
    height: 60px;
    background-color: #ffffff;
    position: absolute;
    top: 5px;
    left: 10px;
    z-index: 2;
    border-radius: 6px;
}

.folder .front {
    background-image: linear-gradient(to left, #24d5ff 0%, #1eb2ff 45%, #1890ff 100%);
    border-radius: 6px;
    z-index: 3;
    box-shadow: 0 1px rgba(255, 255, 255, 0.25) inset, 0 -2px 2px rgba(0, 0, 0, 0.1);
    transform: rotateX(-30deg);
    transform-origin: bottom;
    transition: all .2s ease;

}

.folder:hover .front {
    transform: rotateX(-40deg);
}

.folder .back:before {
    content: " ";
    position: absolute;
    left: 0;
    top: -10px;
    width: 40px;
    height: 10px;
    border-radius: 6px 6px 0 0;
    background-color: #1eb2ff;
}

.folder .back {
    background-image: linear-gradient(to top, #24d5ff 0%, #1eb2ff 45%, #1eb2ff 100%);
    border-radius: 0 6px 6px 6px;
    box-shadow: 0 -1px 1px rgba(0, 0, 0, 0.15);
}

.folderName {
    margin-top: 5px;
    font-size: 14px;
    font-weight: 500;
    text-align: center;
    vertical-align: middle;

    color: rgb(10, 218, 124);
}

.folderTime {
    width: 100%;
    text-align: center;
    font-size: 12px;
    line-height: 1.6;
    color: rgba(25, 165, 23, 0.36);
    max-width: 100%;
    overflow: hidden;
    white-space: nowrap;
    -o-text-overflow: ellipsis;
    text-overflow: ellipsis;
}

.file-icon {
    width: 100px;
    display: flex;
    align-content: space-around;
    flex-wrap: wrap;
}
</style>