<template>
  <div class="p-6 max-w-4xl mx-auto">
    <h1 class="text-2xl font-bold mb-6 text-gray-800">Hardware & Network Settings</h1>

    <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200 mb-6">
      <h2 class="text-xl font-semibold mb-2 text-gray-800">Kiosk Network Access</h2>
      <p class="mb-4 text-gray-600">Enter this URL into the browser of your tablets or kiosk devices connected to the same Wi-Fi network.</p>
      
      <div class="flex items-center space-x-4">
        <input 
          type="text" 
          readonly 
          :value="serverUrl" 
          class="border border-gray-300 p-2 rounded w-full md:w-2/3 bg-gray-50 text-gray-800 font-mono" 
        />
        <button 
          @click="copyUrl" 
          class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded transition-colors"
        >
          Copy URL
        </button>
      </div>
    </div>

    <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
      <h2 class="text-xl font-semibold mb-2 text-gray-800">Thermal Printer Configuration</h2>
      <p class="mb-4 text-gray-600">Select the active POS printer used for printing receipts and sending orders to the grill station.</p>
      
      <div class="flex flex-col space-y-4 md:w-2/3">
        <select 
          v-model="selectedPrinter" 
          class="border border-gray-300 p-2 rounded text-gray-800 bg-white"
        >
          <option disabled value="">Please select a printer</option>
          <option v-for="printer in availablePrinters" :key="printer" :value="printer">
            {{ printer }}
          </option>
        </select>
        
        <button 
          @click="savePrinter" 
          :disabled="!selectedPrinter"
          class="bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white px-4 py-2 rounded w-48 transition-colors"
        >
          Save Printer
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const serverUrl = ref('Loading network data...');
const availablePrinters = ref<string[]>([]);
const selectedPrinter = ref('');

onMounted(async () => {
  try {
    // Fetch network IP
    serverUrl.value = await invoke('get_server_url');
    
    // Fetch available printers
    availablePrinters.value = await invoke('get_available_printers');
    
    // Fetch currently saved printer
    const savedPrinter = await invoke<string>('get_active_printer');
    if (savedPrinter && availablePrinters.value.includes(savedPrinter)) {
      selectedPrinter.value = savedPrinter;
    }
  } catch (error) {
    console.error("Error loading hardware settings:", error);
    serverUrl.value = "Error fetching network IP. Check connection.";
  }
});

const copyUrl = async () => {
  try {
    await navigator.clipboard.writeText(serverUrl.value);
    alert("Kiosk URL copied to clipboard.");
  } catch (err) {
    console.error("Failed to copy URL:", err);
  }
};

const savePrinter = async () => {
  try {
    await invoke('save_active_printer', { printerName: selectedPrinter.value });
    alert("Printer configuration saved successfully.");
  } catch (error) {
    console.error("Failed to save printer:", error);
    alert("An error occurred while saving the printer configuration.");
  }
};
</script>