<template>
  <div>
    <div
      class="text-white text-center font-bold p-4 rounded mb-4"
      v-if="registration_show_alert"
      :class="registration_alert_variant"
    >
      {{ registration_alert_message }}
    </div>
    <form @submit="onSubmit">
      <!-- Name -->
      <div class="mb-3">
        <label class="inline-block mb-2">Name</label>
        <input
          type="text"
          v-bind="name"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
          placeholder="Enter Name"
        />
        <div class="text-red-600">{{ errors.name }}</div>
      </div>
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
      <!-- Age -->
      <div class="mb-3">
        <label class="inline-block mb-2">Age</label>
        <input
          type="number"
          v-bind="age"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
        />
        <div class="text-red-600">{{ errors.age }}</div>
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
      <!-- Confirm Password -->
      <div class="mb-3">
        <label class="inline-block mb-2">Confirm Password</label>
        <input
          type="password"
          v-bind="passwordConfirmation"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
          placeholder="Confirm Password"
        />
        <div class="text-red-600">{{ errors.passwordConfirmation }}</div>
      </div>
      <!-- Country -->
      <div class="mb-3">
        <label class="inline-block mb-2">Country</label>
        <select
          v-bind="country"
          class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
        >
          <option value="USA">USA</option>
          <option value="Mexico">Mexico</option>
          <option value="Germany">Germany</option>
        </select>
        <div class="text-red-600">{{ errors.country }}</div>
      </div>
      <!-- TOS -->
      <div class="mb-3 pl-6">
        <input
          type="checkbox"
          v-bind="termsOfService"
          value="true"
          class="w-4 h-4 float-left -ml-6 mt-1 rounded"
        />
        <label class="inline-block">Accept terms of service</label>
        <div class="text-red-600">{{ errors.termsOfService }}</div>
      </div>

      <button
        type="submit"
        class="block w-full bg-purple-600 text-white py-1.5 px-3 rounded transition hover:bg-purple-700"
        :disable="registration_pending"
      >
        Submit
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useForm } from 'vee-validate'
import { object, string, number, bool, ref as yupRef } from 'yup'
// import { useAuthStore } from '../stores/Auth'
import AuthService from '../api/AuthService'

const { errors, handleSubmit, defineInputBinds } = useForm({
  validationSchema: object({
    name: string().required('Please enter your name'),
    email: string().email().required('Please enter your email'),
    age: number().min(18).max(120).required('Please enter your age'),
    password: string()
      .required('Please Enter your password')
      .matches(
        /^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{8,}$/,
        'Must Contain 8 Characters, One Uppercase, One Lowercase, One Number and one special case Character'
      ),
    passwordConfirmation: string().oneOf([yupRef('password')], 'Passwords does not match'),
    country: string().required(),
    termsOfService: bool().default(false).oneOf([true], 'You must accept the terms of services')
  })
})

// const authStore = useAuthStore()
// Creates a submission handler
// It validate all fields and doesn't call your function unless all fields are valid
const onSubmit = handleSubmit(async (values) => {
  registration_show_alert.value = true
  registration_pending.value = true
  registration_alert_variant.value = 'bg-blue-500'
  registration_alert_message.value = 'Please wait while we create your account'
  console.log('AppRegister::handleSubmit')
  console.log(JSON.stringify(values, null, 2))
  let data = new Map<string, any>([
    ['username', values.name ],
    ['email', values.email ],
    ['password', values.password]
  ])
  let resp = null
  try {
    resp = await AuthService.register(data)
  } catch (error) {
    registration_pending.value = false
    registration_alert_variant.value = 'bg-red-500'
    registration_alert_message.value = 'Failure, an unexpected error occured, please try again later.'
    return
  }
  if ( resp?.data.status === 'fail' ) {
    console.log('response data fail: ' + JSON.stringify(resp?.data, null, 2))
    registration_alert_variant.value = 'bg-red-500'
    registration_alert_message.value = 'Failure: ' + resp?.data.message
  } else {
    console.log('response data success: ' + JSON.stringify(resp?.data, null, 2))
    registration_alert_variant.value = 'bg-green-500'
    registration_alert_message.value = 'Success, your account has been created!'
  }
})

const name = defineInputBinds('name')
const email = defineInputBinds('email')
const age = defineInputBinds('age')
const password = defineInputBinds('password')
const passwordConfirmation = defineInputBinds('passwordConfirmation')
const country = defineInputBinds('country')
const termsOfService = defineInputBinds('termsOfService')

const registration_pending = ref(false)
const registration_show_alert = ref(false)
const registration_alert_variant = ref('bg-blue-500')
const registration_alert_message = ref('Please wait while we create your account')
</script>
