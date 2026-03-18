<template>
  <div class="space-y-3">
    <!-- 标签输入 -->
    <div class="flex gap-2">
      <input
        v-model="newTag"
        placeholder="添加标签..."
        class="flex-1 px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/40 focus:outline-none focus:border-[#0db7ed] text-sm"
        @keyup.enter="addTag"
      />
      <button
        @click="addTag"
        class="px-3 py-2 bg-[#0db7ed]/20 border border-[#0db7ed]/30 text-[#0db7ed] rounded-lg hover:bg-[#0db7ed]/30 transition-all text-sm font-medium"
      >
        添加
      </button>
    </div>
    
    <!-- 现有标签 -->
    <div v-if="tags.length > 0" class="flex flex-wrap gap-2">
      <div
        v-for="tag in tags"
        :key="tag"
        class="flex items-center gap-1 px-2 py-1 bg-white/5 border border-white/10 rounded-full text-white/70 text-xs"
      >
        <span>{{ tag }}</span>
        <button
          @click="removeTag(tag)"
          class="text-white/40 hover:text-red-400 transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTagStore } from "@/stores/modules/tagStore";

const props = defineProps<{
    resourceId: string;
}>();

const tagStore = useTagStore();
const newTag = ref("");

const tags = computed(() => {
    return tagStore.getTagsByResourceId(props.resourceId);
});

const addTag = () => {
    if (newTag.value.trim()) {
        tagStore.addTagToResource(props.resourceId, newTag.value.trim());
        newTag.value = "";
    }
};

const removeTag = (tag: string) => {
    tagStore.removeTagFromResource(props.resourceId, tag);
};
</script>
