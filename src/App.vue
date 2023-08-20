<template>
  <div id="app-view">
    <div id="side-bar">
      <el-menu>
        <el-menu-item index="1-1">文件管理</el-menu-item>
        <el-menu-item index="1-2">转码任务</el-menu-item>
        <el-menu-item index="1-3">传输任务</el-menu-item>
      </el-menu>
    </div>
    <div id="main-pane">
      <Filemanager></Filemanager>
    </div>
  </div>
</template>

<script setup lang="ts">
import Filemanager from "./views/Filemanager.vue";
</script>


<style scoped>
#app-view {
  width: 100%;
  min-height: 100vh;
  display: flex;
}

#side-bar {
  height: 100%;
  width: 6em;
  flex-grow: 0;
}

#main-pane {
  border: solid gray 1px;
  flex-grow: 1;
}

.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
