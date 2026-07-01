<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { scheduleService, type Shift } from '../../services/scheduleService';
import DataLoader from '../ui/DataLoader.vue';
import { useResponsive } from '../../composables/useResponsive';

const { fontSm, fontBase, fontLg, fontXl } = useResponsive();

const props = defineProps<{
  isOpen: boolean;
  staff: any;
}>();

// const emit = defineEmits<{
//   (e: 'close'): void;
// }>();

const shifts = ref<Shift[]>([]);
const isLoading = ref(false);

watch(() => props.isOpen, async (newVal) => {
  if (newVal && props.staff) {
    isLoading.value = true;
    try {
      shifts.value = await scheduleService.getStaffShifts(props.staff.staff_id);
    } catch (error) {
      console.error("Failed to load timesheet:", error);
    } finally {
      isLoading.value = false;
    }
  } else {
    shifts.value = [];
  }
});

const totalHoursRendered = computed(() => {
  return shifts.value.reduce((total, shift) => {
    return total + (shift.total_rendered_hours || 0);
  }, 0);
});

function formatTime(datetimeStr: string | null) {
  if (!datetimeStr) return '--:--';
  const date = new Date(datetimeStr.replace(' ', 'T'));
  return date.toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
}
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-md" @click.self="$emit('close')">
    <div class="bg-surface-container-low border border-outline-variant/20 w-full max-w-3xl rounded-2xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden">
      
      <div class="p-6 border-b border-outline-variant/10 bg-surface-container-highest/30 flex justify-between items-start shrink-0">
        <div>
          <h3 :class="['font-black text-on-surface tracking-tight', fontXl]">{{ staff?.full_name }}</h3>
          <p :class="['font-medium text-on-surface-variant mt-1', fontSm]">Rendered Work History (Last 30 Shifts)</p>
        </div>
        <button @click="$emit('close')" class="text-on-surface-variant hover:text-on-surface bg-surface-container hover:bg-surface-container-high p-2 rounded-full transition-colors active:scale-90">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-6 bg-surface">
        <div v-if="isLoading" class="py-12 flex justify-center">
          <DataLoader message="Loading timesheet data..." />
        </div>

        <div v-else-if="shifts.length === 0" class="py-12 text-center text-on-surface-variant">
          <svg class="w-12 h-12 mx-auto mb-3 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          <p :class="['font-medium', fontBase]">No shift history found for this employee.</p>
        </div>

        <div v-else class="border border-outline-variant/15 rounded-xl overflow-hidden">
          <table class="w-full text-left border-collapse bg-surface-container-low">
            <thead class="bg-surface-container sticky top-0 z-10">
              <tr :class="['border-b border-outline-variant/20 text-on-surface-variant uppercase tracking-widest', fontSm]">
                <th class="p-4 font-bold">Shift Date</th>
                <th class="p-4 font-bold">Time In</th>
                <th class="p-4 font-bold">Time Out</th>
                <th class="p-4 font-bold text-right">Hours</th>
              </tr>
            </thead>
            <tbody class="text-on-surface">
              <tr v-for="shift in shifts" :key="shift.shift_id" class="border-b border-outline-variant/10 hover:bg-surface-container-high transition-colors">
                <td :class="['p-4 font-bold', fontSm]">{{ new Date(shift.shift_date).toLocaleDateString() }}</td>
                <td :class="['p-4 text-tertiary font-bold', fontSm]">{{ formatTime(shift.clock_in_time) }}</td>
                <td :class="['p-4 text-error font-bold', fontSm]">{{ formatTime(shift.clock_out_time) }}</td>
                <td :class="['p-4 font-black text-primary-container text-right', fontSm]">
                  {{ shift.total_rendered_hours ? shift.total_rendered_hours.toFixed(2) + ' h' : 'Active' }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div class="p-6 bg-surface-container-highest/20 border-t border-outline-variant/10 shrink-0 flex justify-between items-center">
        <span :class="['font-bold text-on-surface-variant uppercase tracking-widest', fontSm]">Total Rendered (Last 30)</span>
        <span :class="['font-black text-primary', fontLg]">{{ totalHoursRendered.toFixed(2) }} Hours</span>
      </div>

    </div>
  </div>
</template>