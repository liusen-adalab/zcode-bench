import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";


const app = createApp(App)

import ElementPlus from 'element-plus'
app.use(ElementPlus)

import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
    app.component(key, component)
}

import "@imengyu/vue3-context-menu/lib/vue3-context-menu.css";
import ContextMenu from "@imengyu/vue3-context-menu";
app.use(ContextMenu);

app.mount("#app")

