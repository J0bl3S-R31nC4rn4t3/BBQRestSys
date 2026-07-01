<script setup lang="ts">
import { ref, onMounted } from "vue";
import { staffService } from "../services/staffService";
import { useAuth } from "../stores/authStore";
import Header from "../components/layout/Header.vue";
import StaffModal from "../components/modal/StaffModal.vue";
import StaffTimesheetModal from "../components/modal/StaffTimesheetModal.vue";
import DataLoader from "../components/ui/DataLoader.vue";
import BaseButton from "../components/ui/BaseButton.vue";
import { useResponsive } from '../composables/useResponsive';

const { fontSm, fontBase } = useResponsive();
const authStore = useAuth();

const staffMembers = ref<any[]>([]);
const isModalOpen = ref(false);
const isTimesheetModalOpen = ref(false);
const selectedStaff = ref<any>(null);

const isLoadingData = ref(true);

async function loadStaff() {
  isLoadingData.value = true;
  try {
    const result = await staffService.getAllStaff();
    let staffList = result as any[];

    if (authStore.currentUser.value?.role !== 'Super Admin') {
      staffList = staffList.filter(s => s.role !== 'Super Admin');
    }

    staffMembers.value = staffList;
  } catch (error) {
    console.error("Failed to load staff:", error);
  } finally {
    setTimeout(() => {
      isLoadingData.value = false;
    }, 600);
  }
}

async function handleSaveStaff(staffData: any) {
  try {
    if (staffData.id) {
      await staffService.updateStaff(staffData.id, staffData);
    } else {
      await staffService.createStaff(staffData);
    }

    closeModal();
    await loadStaff(); 
  } catch (error) {
    console.error("Failed to save staff:", error);
    alert("Error saving staff member.");
  }
}

async function handleDeleteStaff(id: number, name: string) {
  const isConfirmed = confirm(`Are you sure you want to permanently delete ${name}?`);

  if (isConfirmed) {
    try {
      await staffService.deleteStaff(id);
      await loadStaff(); 
    } catch (error) {
      console.error("Failed to delete staff:", error);
      alert("Error: Cannot delete this staff member because they are linked to existing transactions or prep logs.");
    }
  }
}

function openModal(staff?: any) {
  selectedStaff.value = staff || null;
  isModalOpen.value = true;
}

function openTimesheetModal(staff: any) {
  selectedStaff.value = staff;
  isTimesheetModalOpen.value = true;
}

function closeModal() {
  isModalOpen.value = false;
  isTimesheetModalOpen.value = false;
  selectedStaff.value = null;
}

onMounted(() => {
  loadStaff();
});
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="bg-surface-container-low p-4 md:p-6 rounded-2xl shadow-sm border border-outline-variant/15 flex-1 flex flex-col min-h-0 overflow-hidden">

      <Header 
        title="Staff Directory" 
        subtitle="Manage employee records and system access"
        customClass="mb-4"
      >
        <template #actions>
          <BaseButton variant="primary" @click="openModal()" :class="['w-full md:w-auto flex justify-center py-2.5 px-4 h-9.5', fontBase]">
            <svg class="w-4 h-4 mr-2 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
            </svg>
            <span class="tracking-widest uppercase text-xs font-bold">Add Staff Member</span>
          </BaseButton>
        </template>
      </Header>

      <DataLoader v-if="isLoadingData" message="Loading staff records..." class="m-auto" />

      <div v-else class="flex-1 overflow-auto bg-surface rounded-xl border border-outline-variant/15">
        <table class="w-full text-left border-collapse min-w-150 lg:min-w-full">
          <thead class="bg-surface-container sticky top-0 z-10 shadow-sm outline outline-outline-variant/10">
            <tr :class="['text-on-surface-variant uppercase tracking-widest font-bold', fontSm]">
              <th class="py-3 px-3 sm:py-4 sm:px-4">Name</th>
              <th class="py-3 px-3 sm:py-4 sm:px-4">Role</th>
              <th class="py-3 px-3 sm:py-4 sm:px-4">Contact Number</th>
              <th class="py-3 px-3 sm:py-4 sm:px-4">Account Status</th>
              <th class="py-3 px-3 sm:py-4 sm:px-4 text-right">Actions</th>
            </tr>
          </thead>
          <tbody class="text-on-surface divide-y divide-outline-variant/10">
            <tr v-if="staffMembers.length === 0">
              <td colspan="5" class="py-8 sm:py-12 text-center text-on-surface-variant font-bold uppercase tracking-widest text-xs sm:text-sm">No staff records found. Add one to begin.</td>
            </tr>
            <tr v-for="staff in staffMembers" :key="staff.staff_id" class="hover:bg-surface-container-high transition-colors">
              <td :class="['py-3 px-3 sm:py-4 sm:px-4 font-black text-on-surface', fontBase]">{{ staff.full_name }}</td>
              <td :class="['py-3 px-3 sm:py-4 sm:px-4 text-on-surface-variant font-bold', fontBase]">{{ staff.role }}</td>
              <td :class="['py-3 px-3 sm:py-4 sm:px-4 text-on-surface-variant font-medium', fontBase]">{{ staff.phone_number || 'N/A' }}</td>
              <td class="py-3 px-3 sm:py-4 sm:px-4">
                <span :class="staff.status === 'Active' ? 'bg-tertiary/10 text-tertiary border-tertiary/20' : 'bg-surface-container border-outline-variant/30 text-on-surface-variant'" class="px-2 py-1 sm:px-3 sm:py-1 border rounded-full text-[9px] sm:text-[10px] font-bold uppercase tracking-wider">
                  {{ staff.status }}
                </span>
              </td>
              <td class="py-3 px-3 sm:py-4 sm:px-4 text-right">
                <div v-if="staff.role !== 'Super Admin'" class="flex justify-end gap-1.5 sm:gap-3">
                  <button @click="openTimesheetModal(staff)" class="p-1.5 sm:p-2 border border-secondary-container/30 text-on-secondary-container rounded-lg bg-secondary-container/10 hover:bg-secondary-container/20 transition-colors active:scale-90" title="View Rendered Hours">
                    <svg class="w-4 h-4 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                  </button>

                  <button @click="openModal(staff)" class="p-1.5 sm:p-2 border border-primary/20 text-primary rounded-lg bg-primary/10 hover:bg-primary/20 transition-colors active:scale-90" title="Edit">
                    <svg class="w-4 h-4 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"></path></svg>
                  </button>

                  <button @click="handleDeleteStaff(staff.staff_id, staff.full_name)" class="p-1.5 sm:p-2 border border-error/20 text-error rounded-lg bg-error/10 hover:bg-error/20 transition-colors active:scale-90" title="Delete">
                    <svg class="w-4 h-4 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg>
                  </button>
                </div>
                <div v-else class="flex justify-end">
                  <span class="bg-surface-container text-on-surface-variant text-[9px] sm:text-[10px] font-bold px-2 py-1 sm:px-3 sm:py-1.5 rounded-md border border-outline-variant/20 uppercase tracking-widest cursor-not-allowed">
                    System Protected
                  </span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

    </div>

    <StaffModal :isOpen="isModalOpen" :staffData="selectedStaff" @close="closeModal" @save="handleSaveStaff" />
    <StaffTimesheetModal :isOpen="isTimesheetModalOpen" :staff="selectedStaff" @close="closeModal" />

  </div>
</template>