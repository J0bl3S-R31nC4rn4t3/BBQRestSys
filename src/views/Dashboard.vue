<script setup lang="ts">
import { ref, onMounted, shallowRef, watch, nextTick } from 'vue';
import { dashboardService, type PeriodMetrics } from '../services/dashboardService';
import Header from '../components/layout/Header.vue';
import DataLoader from '../components/ui/DataLoader.vue';
import Chart from 'chart.js/auto';
import { useResponsive } from '../composables/useResponsive';

const { fontSm, fontBase, fontLg, fontXl,} = useResponsive();

const isLoadingData = ref(true);
const isUpdatingChart = ref(false);
const isLoadingTopItems = ref(false);

const todaySales = ref(0);
const activeStaff = ref(0);
const lowStockAlerts = ref<any[]>([]);
const topItems = ref<any[]>([]);

const selectedPeriod = ref<'daily' | 'weekly' | 'monthly' | 'yearly'>('weekly');
const topItemsPeriod = ref<'daily' | 'weekly' | 'monthly' | 'yearly'>('daily');

const salesChartCanvas = ref<HTMLCanvasElement | null>(null);
const meatChartCanvas = ref<HTMLCanvasElement | null>(null);

const salesChart = shallowRef<Chart | null>(null);
const meatChart = shallowRef<Chart | null>(null);

const currentMetrics = ref<PeriodMetrics>({
  current_sales: 0,
  previous_sales: 0,
  skewers_sold: 0,
  orders: [],
  meat_distribution: []
});

const trendHtml = ref("");
const chartLabelText = ref("");

async function loadDashboard() {
  isLoadingData.value = true;
  try {
    const [sales, staff, alerts] = await Promise.all([
      dashboardService.getTodaySales(),
      dashboardService.getActiveStaffCount(),
      dashboardService.getLowStockAlerts()
    ]);
    todaySales.value = sales;
    activeStaff.value = staff;
    lowStockAlerts.value = alerts;
    
    await Promise.all([
      fetchPeriodMetrics(),
      fetchTopItems()
    ]);
  } catch (error) {
    console.error("Error loading dashboard metrics:", error);
  } finally {
    setTimeout(async () => {
      isLoadingData.value = false;
      await nextTick();
      initCharts();
    }, 600);
  }
}

async function fetchPeriodMetrics() {
  isUpdatingChart.value = true;
  try {
    currentMetrics.value = await dashboardService.getPeriodMetrics(selectedPeriod.value);
    updateTrendCalculation();
    if (salesChart.value && meatChart.value) {
      updateCharts();
    }
  } catch (error) {
    console.error("Error fetching period metrics:", error);
  } finally {
    isUpdatingChart.value = false;
  }
}

async function fetchTopItems() {
  isLoadingTopItems.value = true;
  try {
    // Requires updating dashboardService.ts and backend to accept the period parameter
    topItems.value = await dashboardService.getTopSellingItems(topItemsPeriod.value);
  } catch (error) {
    console.error("Error fetching top items:", error);
  } finally {
    isLoadingTopItems.value = false;
  }
}

function updateTrendCalculation() {
  const current = currentMetrics.value.current_sales;
  const previous = currentMetrics.value.previous_sales;
  
  if (previous === 0) {
    trendHtml.value = "<span class='text-tertiary font-bold'>100% Increase</span> vs previous";
  } else {
    const diff = current - previous;
    const percent = (Math.abs(diff) / previous) * 100;
    
    if (diff >= 0) {
      trendHtml.value = `<span class='text-tertiary font-bold'>Up ${percent.toFixed(1)}%</span> vs previous`;
    } else {
      trendHtml.value = `<span class='text-error font-bold'>Down ${percent.toFixed(1)}%</span> vs previous`;
    }
  }

  chartLabelText.value = selectedPeriod.value === 'daily' ? 'Today' 
                       : selectedPeriod.value === 'weekly' ? 'This Week' 
                       : selectedPeriod.value === 'monthly' ? 'This Month' : 'This Year';
}

function processChartData() {
  const labels: string[] = [];
  const data: number[] = [];
  const grouped = new Map<string, number>();

  currentMetrics.value.orders.forEach(order => {
    const date = new Date(order.timestamp);
    let key = "";
    
    if (selectedPeriod.value === 'daily') {
      key = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (selectedPeriod.value === 'weekly' || selectedPeriod.value === 'monthly') {
      key = date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    } else {
      key = date.toLocaleDateString([], { month: 'short', year: 'numeric' });
    }
    
    grouped.set(key, (grouped.get(key) || 0) + order.amount);
  });

  grouped.forEach((value, key) => {
    labels.push(key);
    data.push(value);
  });

  return { labels, data };
}

function processMeatData() {
  const labels = currentMetrics.value.meat_distribution.map(d => d.category);
  const data = currentMetrics.value.meat_distribution.map(d => Number(d.quantity));
  
  const colors = ['#ff6d00', '#ffb692', '#f0be74', '#bf924c', '#ddc1b7', '#56423b'];
  const bgColors = labels.map((_, i) => colors[i % colors.length]);

  const total = data.reduce((a, b) => a + b, 0);

  return { labels, data, bgColors, total };
}

function initCharts() {
  if (!salesChartCanvas.value || !meatChartCanvas.value) return;

  const salesRes = processChartData();
  salesChart.value = new Chart(salesChartCanvas.value, {
    type: 'line',
    data: {
      labels: salesRes.labels,
      datasets: [{
        label: 'Gross Sales (PHP)',
        data: salesRes.data,
        borderColor: '#ffb692',
        backgroundColor: 'rgba(255, 182, 146, 0.1)',
        borderWidth: 2,
        fill: true,
        tension: 0.4
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: { legend: { display: false } },
      layout: { padding: { left: 0, bottom: 0 } },
      scales: { 
        y: { 
          beginAtZero: true,
          grid: { color: 'rgba(255, 255, 255, 0.05)' },
          ticks: { color: '#e6e1df', font: { size: 10 } }
        },
        x: {
          grid: { color: 'rgba(255, 255, 255, 0.05)' },
          ticks: { color: '#e6e1df', font: { size: 10 }, maxRotation: 45, minRotation: 45 }
        }
      }
    }
  });

  const meatRes = processMeatData();
  meatChart.value = new Chart(meatChartCanvas.value, {
    type: 'doughnut',
    data: {
      labels: meatRes.labels.length > 0 ? meatRes.labels : ['No Data'],
      datasets: [{
        data: meatRes.data.length > 0 ? meatRes.data : [1],
        backgroundColor: meatRes.bgColors.length > 0 ? meatRes.bgColors : ['#363433'],
        borderWidth: 0
      }]
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      cutout: '70%',
      layout: { padding: 0 },
      plugins: { legend: { display: false } }
    }
  });
}

function updateCharts() {
  if (!salesChart.value || !meatChart.value) return;

  const salesRes = processChartData();
  salesChart.value.data.labels = salesRes.labels;
  salesChart.value.data.datasets[0].data = salesRes.data;
  salesChart.value.update();

  const meatRes = processMeatData();
  meatChart.value.data.labels = meatRes.labels.length > 0 ? meatRes.labels : ['No Data'];
  meatChart.value.data.datasets[0].data = meatRes.data.length > 0 ? meatRes.data : [1];
  meatChart.value.data.datasets[0].backgroundColor = meatRes.bgColors.length > 0 ? meatRes.bgColors : ['#363433'];
  meatChart.value.update();
}

watch(selectedPeriod, () => {
  fetchPeriodMetrics();
});

watch(topItemsPeriod, () => {
  fetchTopItems();
});

onMounted(() => {
  loadDashboard();
});
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Main Scrollable Container -->
    <div class="flex-1 overflow-y-auto space-y-4 pb-6">

      <!-- Header Always Visible -->
      <Header 
        title="Business Overview" 
        isSticky 
        isGlass 
        customClass="-mt-4 pt-4 pb-3 mb-4 border-b border-outline-variant/20 md:pl-3"
      >
        <template #actions>
          <div class="flex bg-surface-container-low rounded-lg shadow-sm border border-outline-variant/30 p-1 w-full md:w-auto">
            <button @click="selectedPeriod = 'daily'" :class="[selectedPeriod === 'daily' ? 'bg-primary-container/20 text-primary font-bold' : 'text-on-surface-variant hover:bg-surface-container', 'flex-1 md:flex-none px-3 py-1 font-medium rounded transition-colors uppercase tracking-widest text-[10px]']">Daily</button>
            <button @click="selectedPeriod = 'weekly'" :class="[selectedPeriod === 'weekly' ? 'bg-primary-container/20 text-primary font-bold' : 'text-on-surface-variant hover:bg-surface-container', 'flex-1 md:flex-none px-3 py-1 font-medium rounded transition-colors uppercase tracking-widest text-[10px]']">Weekly</button>
            <button @click="selectedPeriod = 'monthly'" :class="[selectedPeriod === 'monthly' ? 'bg-primary-container/20 text-primary font-bold' : 'text-on-surface-variant hover:bg-surface-container', 'flex-1 md:flex-none px-3 py-1 font-medium rounded transition-colors uppercase tracking-widest text-[10px]']">Monthly</button>
            <button @click="selectedPeriod = 'yearly'" :class="[selectedPeriod === 'yearly' ? 'bg-primary-container/20 text-primary font-bold' : 'text-on-surface-variant hover:bg-surface-container', 'flex-1 md:flex-none px-3 py-1 font-medium rounded transition-colors uppercase tracking-widest text-[10px]']">Yearly</button>
          </div>
        </template>
      </Header>

      <!-- Loading State: Rendered below the header -->
      <div v-if="isLoadingData" class="flex flex-col items-center justify-center min-h-[50vh] bg-surface-container-low rounded-xl shadow-sm border border-outline-variant/15 mx-1">
        <DataLoader message="Compiling live metrics..." />
      </div>

      <!-- Dashboard Data Grid: Rendered below the header once loaded -->
      <div v-else class="grid grid-cols-12 gap-4 mx-1">
        
        <div class="col-span-12 grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="bg-surface-container-low p-4 rounded-xl shadow-sm border border-outline-variant/15 flex items-center gap-3">
            <div class="w-10 h-10 bg-tertiary/10 text-tertiary rounded-full flex items-center justify-center border border-tertiary/20 shrink-0">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            </div>
            <div>
              <p :class="['font-bold text-on-surface-variant uppercase tracking-widest text-[9px] leading-tight mb-0.5',fontSm]">Live Sales (Today)</p>
              <h3 :class="['font-black text-on-surface leading-none', fontXl]">₱{{ todaySales.toFixed(2) }}</h3>
            </div>
          </div>

          <div class="bg-surface-container-low p-4 rounded-xl shadow-sm border border-outline-variant/15 flex items-center gap-3">
            <div class="w-10 h-10 bg-primary-container/20 text-primary rounded-full flex items-center justify-center border border-primary-container/30 shrink-0">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path></svg>
            </div>
            <div>
              <p class="font-bold text-on-surface-variant uppercase tracking-widest text-[9px] leading-tight mb-0.5">Live Active Staff</p>
              <h3 :class="['font-black text-on-surface leading-none', fontXl]">{{ activeStaff }} Members</h3>
            </div>
          </div>

          <div :class="[lowStockAlerts.length > 0 ? 'bg-error/10 border-error/30' : 'bg-surface-container-low border-outline-variant/15', 'p-4 rounded-xl shadow-sm border flex items-center gap-3 transition-colors']">
            <div :class="[lowStockAlerts.length > 0 ? 'bg-error/20 text-error border-error/30' : 'bg-surface-container text-on-surface-variant border-outline-variant/20', 'w-10 h-10 rounded-full flex items-center justify-center border shrink-0']">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
            </div>
            <div>
              <p :class="['font-bold uppercase tracking-widest text-[9px] leading-tight mb-0.5', lowStockAlerts.length > 0 ? 'text-error' : 'text-on-surface-variant']">Inventory Warnings</p>
              <h3 :class="['font-black leading-none', lowStockAlerts.length > 0 ? 'text-error' : 'text-on-surface', fontXl]">{{ lowStockAlerts.length }} Critical</h3>
            </div>
          </div>
        </div>

        <div class="col-span-12 lg:col-span-8 relative bg-surface-container-low p-4 rounded-xl shadow-sm border border-outline-variant/15 flex flex-col">
          <div v-if="isUpdatingChart" class="absolute inset-0 z-10 bg-surface/50 backdrop-blur-sm rounded-xl flex items-center justify-center">
            <p class="font-black text-primary uppercase tracking-widest animate-pulse text-xs">Updating...</p>
          </div>
          
          <div class="flex justify-between items-start mb-4">
            <div>
              <h3 :class="['font-black text-on-surface tracking-tight leading-none', fontLg]">Sales</h3>
              <p class="text-[10px] mt-1.5" v-html="trendHtml"></p>
            </div>
            <div class="text-right">
              <span class="text-[9px] text-on-surface-variant font-bold uppercase tracking-widest block mb-0.5">{{ chartLabelText }}</span>
              <p :class="['font-black text-primary leading-none', fontLg]">₱{{ currentMetrics.current_sales.toFixed(2) }}</p>
            </div>
          </div>
          <div class="relative h-48 w-full flex-1">
            <canvas ref="salesChartCanvas"></canvas>
          </div>
        </div>

        <div class="col-span-12 lg:col-span-4 bg-surface-container-low p-4 rounded-xl shadow-sm border border-outline-variant/15 flex flex-col">
          <div class="flex justify-between items-start mb-4">
            <h3 :class="['font-black text-on-surface tracking-tight leading-none', fontLg]">Volume</h3>
            <div class="text-right">
              <p :class="['font-black text-primary-container leading-none mb-0.5', fontBase]">{{ currentMetrics.skewers_sold }}</p>
              <span class="text-[9px] text-on-surface-variant font-bold uppercase tracking-widest">Units</span>
            </div>
          </div>
          
          <div class="flex flex-row md:flex-col items-center gap-4 flex-1">
            <div class="relative h-32 w-32 shrink-0">
              <canvas ref="meatChartCanvas"></canvas>
            </div>
            
            <div class="flex-1 w-full space-y-2 overflow-y-auto max-h-32 pr-1 custom-scrollbar">
              <div v-if="currentMetrics.meat_distribution.length === 0" class="text-center text-xs text-on-surface-variant font-medium py-2">
                No data.
              </div>
              <div v-for="(dist, index) in currentMetrics.meat_distribution" :key="index" class="flex justify-between text-xs items-center bg-surface-container/50 px-2 py-1.5 rounded">
                <span class="text-on-surface font-bold truncate pr-2">{{ dist.category }}</span>
                <span class="font-black text-primary-container shrink-0">{{ dist.quantity }}x</span>
              </div>
            </div>
          </div>
        </div>

        <div class="col-span-12 bg-surface-container-low p-4 rounded-xl shadow-sm border border-outline-variant/15 relative">
          <div v-if="isLoadingTopItems" class="absolute inset-0 z-10 bg-surface/50 backdrop-blur-sm rounded-xl flex items-center justify-center">
            <p class="font-black text-primary uppercase tracking-widest animate-pulse text-xs">Fetching...</p>
          </div>

          <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-4 gap-2">
            <h3 :class="['font-black text-on-surface tracking-tight leading-none', fontLg]">Top Performing Items</h3>
            <select v-model="topItemsPeriod" class="bg-surface-container border border-outline-variant/30 text-on-surface text-[10px] font-bold uppercase tracking-widest rounded-lg px-3 py-1.5 outline-none focus:border-primary-container focus:ring-1 focus:ring-primary-container transition-colors">
              <option value="daily">Today</option>
              <option value="weekly">This Week</option>
              <option value="monthly">This Month</option>
              <option value="yearly">This Year</option>
            </select>
          </div>
          
          <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
            <div v-if="topItems.length === 0" class="col-span-full text-center text-on-surface-variant text-xs py-2">No items sold for this period.</div>
            <div v-for="(item, index) in topItems.slice(0, 5)" :key="index" class="bg-surface-container border border-outline-variant/10 rounded-lg p-2.5 flex items-center justify-between">
              <div class="overflow-hidden pr-2">
                <p class="text-[9px] text-on-surface-variant font-black mb-0.5 uppercase tracking-widest leading-none">#{{ index + 1 }}</p>
                <p class="font-bold text-on-surface text-xs truncate leading-tight" :title="item.pos_display_name">{{ item.pos_display_name }}</p>
              </div>
              <div class="bg-primary-container/20 text-primary border border-primary-container/30 font-black text-xs px-2 py-0.5 rounded shrink-0">
                {{ item.total_sold }}
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(150, 150, 150, 0.2);
  border-radius: 4px;
}
</style>