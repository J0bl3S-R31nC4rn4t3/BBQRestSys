<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import Sidebar from './components/layout/Sidebar.vue';
import LoadingScreen from './components/ui/LoadingScreen.vue';

const isAppReady = ref(false);
const route = useRoute();

const isPublicRoute = computed(() => route.meta.public === true);

onMounted(() => {
  try {
    // Session initialization logic can go here later
  } catch (error) {
    console.error("Error loading user session:", error);
  } finally {
    isAppReady.value = true;
  }
});
</script>

<template>
  <LoadingScreen :is-loading="!isAppReady" />

  <!-- Added data-tauri-drag-region here -->
  <div v-show="isAppReady" data-tauri-drag-region class="flex h-screen bg-surface text-on-surface font-sans overflow-hidden antialiased">
    
    <Sidebar v-if="!isPublicRoute" />

    <main class="flex-1 flex flex-col h-screen relative overflow-hidden">
      
      <div :class="['flex-1 overflow-y-auto', route.path === '/login' ? 'p-0' : 'p-0 md:p-0']">
        <router-view></router-view>
      </div>

    </main>

  </div>
</template>