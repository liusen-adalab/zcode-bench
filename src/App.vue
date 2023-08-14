<script setup lang="ts">
import Filemanager from "./views/Filemanager.vue";
</script>

<template>
  <div class="common-layout">
    <el-container>
      <el-header>Header</el-header>
      <el-container>
        <el-aside width="100px">
          <el-menu>
            <el-menu-item index="1-1">文件管理</el-menu-item>
            <el-menu-item index="1-2">转码任务</el-menu-item>
            <el-menu-item index="1-3">传输任务</el-menu-item>
          </el-menu>
        </el-aside>
        <el-container>
          <el-main class="main-pane">
            <Filemanager></Filemanager>
          </el-main>
        </el-container>
      </el-container>
    </el-container>
  </div>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.main-pane {
  height: 85vh;
  border-style: solid;
  border-color: #249b73;
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
