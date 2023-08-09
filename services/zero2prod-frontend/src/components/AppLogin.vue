<template>
  <div>
    <div
      class="text-white text-center font-bold p-4 rounded mb-4"
      v-if="login_show_alert"
      :class="login_alert_variant"
    >
      {{ login_alert_message }}
    </div>
    <form @submit="onSubmit">
      <!-- Email -->
      <div class="mb-3">
        <label class="inline-block mb-2">Email</label>
        <input
          type="email"
          v-bind="email"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
          placeholder="Enter Email"
        />
        <div class="text-red-600">{{ errors.email }}</div>
      </div>
      <!-- Password -->
      <div class="mb-3">
        <label class="inline-block mb-2">Password</label>
        <input
          type="password"
          v-bind="password"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
          placeholder="Password"
        />
        <div class="text-red-600">{{ errors.password }}</div>
      </div>
      <button
        type="submit"
        class="block w-full bg-purple-600 text-white py-1.5 px-3 rounded transition hover:bg-purple-700"
        :disable="login_pending"
      >
        Submit
      </button>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { useForm } from 'vee-validate'
import { object, string } from 'yup'
import { useAuthStore } from '../stores/Auth'
import { MyError } from '../types/Error'

export default defineComponent({
  setup() {
    const { errors, handleSubmit, defineInputBinds } = useForm({
      validationSchema: object({
        email: string().email().required('Please enter your email'),
        password: string().required('Pleas enter your password')
      })
    })

    const authStore = useAuthStore()
    // Creates a submission handler
    // It validate all fields and doesn't call your function unless all fields are valid
    const onSubmit = handleSubmit(async (values) => {
      login_show_alert.value = true
      login_pending.value = true
      login_alert_variant.value = 'bg-blue-500'
      login_alert_message.value = 'Please wait while we create your account'
      let data = new Map<string, any>([
        ['email', values.email],
        ['password', values.password]
      ])
      try {
        await authStore.login(data)
      } catch (error) {
        login_pending.value = false
        login_alert_variant.value = 'bg-red-500'
        if (error instanceof MyError) {
          login_alert_message.value = error.message
        } else {
          login_alert_message.value =
            'Failure, an unexpected error occured, please try again later.'
        }
        return
      }
      login_alert_variant.value = 'bg-green-500'
      login_alert_message.value = 'Success, you are now logged in'
      window.location.reload()
    })

    const email = defineInputBinds('email')
    const password = defineInputBinds('password')

    const login_pending = ref(false)
    const login_show_alert = ref(false)
    const login_alert_variant = ref('bg-blue-500')
    const login_alert_message = ref('Please wait while we create your account')

    return {
      errors,
      onSubmit,
      email,
      password,
      login_pending,
      login_show_alert,
      login_alert_variant,
      login_alert_message
    }
  }
})
</script>
