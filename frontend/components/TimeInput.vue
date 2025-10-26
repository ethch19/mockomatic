<template>
    <Input
        v-model="value"
        type="text"
        class="text-left text-sm"
        @blur="submitChange"
        @keydown.enter="handleEnter"
    />
</template>

<script setup lang="ts">
import { Input } from '@/components/ui/input'
import { type Time, type TimeDuration, parseTime } from '@internationalized/date'
import z from 'zod';
import { normalizeTimeString } from '~/composables/formatting';

const props = defineProps<{
  modelValue: Time | null
}>()

const emit = defineEmits<{
  (e: "update", duration: TimeDuration): void
}>()

const originalTimeString = computed(() => {
    return props.modelValue ? props.modelValue.toString() : '00:00:00'
})

const { value, errorMessage, validate, resetField } = useField<string>("time",
    toTypedSchema(z.preprocess(
        (val) => normalizeTimeString(val as string),
        z.string().time({ precision: 0 })
    )),
    {
        initialValue: originalTimeString.value,
        validateOnValueUpdate: false
    }
);

watch(originalTimeString, (newString) => {
    resetField({ value: newString })
})

async function submitChange() {
    const { valid, value: normalizedValue } = await validate();

    if (valid) {
        if (normalizedValue !== originalTimeString.value) {
            const newTime = parseTime(normalizedValue as string);
            const originalTime = props.modelValue;

            // stupidly, TimeDuration's properties have an extra 's', so cannot just pass Time as a parameter
            const duration_diff: TimeDuration = {
                hours: newTime.hour - originalTime?.hour,
                minutes: newTime.minute - originalTime?.minute,
                seconds: newTime.second - originalTime?.second,
                milliseconds: newTime.millisecond - originalTime?.millisecond,
            }

            emit('update', duration_diff) // for table to update
        }
    } else {
        resetField({ value: originalTimeString.value })
    }
}

function handleEnter(event: KeyboardEvent) {
    submitChange();
    (event.target as HTMLInputElement).blur()
}
</script>