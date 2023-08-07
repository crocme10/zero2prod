<template>
  <div class="fixed z-10 inset-0 overflow-y-auto" id="modal" :class="hiddenClass">
    <div
      class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
      >
      <div class="fixed inset-0 transition-opacity">
        <div class="absolute inset-0 bg-gray-800 opacity-75"></div>
      </div>

      <!-- This element is to trick the browser into centering the modal contents. -->
      <span class="hidden sm:inline-block sm:align-middle sm:h-screen"
        >&#8203;</span
      >

      <div
        class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full"
        >
        <!-- Add margin if you want to see some of the overlay behind the modal-->
        <div class="py-4 text-left px-6">
          <!--Title-->
        <div class="flex justify-between items-center pb-4">
          <p class="text-2xl font-bold">Your Account</p>
          <!-- Modal Close Button -->
          <div class="modal-close cursor-pointer z-50" @click.prevent="toggleHidden">
            <i class="icon-solid-xmark"></i>
          </div>
        </div>

        <!-- Tabs -->
        <ul class="flex flex-wrap mb-4">
          <li class="flex-auto text-center">
            <a
              class="block rounded py-3 px-4 transition"
              href="#"
              @click.prevent = "tab = 'login'"
              :class="{
                'hover:text-white text-white bg-blue-600': tab === 'login',
                'hover:text-blue-600': tab === 'register'
              }"
              >Login</a
            >
          </li>
              <li class="flex-auto text-center">
                <a class="block rounded py-3 px-4 transition" href="#"
                  @click.prevent = "tab = 'register'"
                  :class="{
                  'hover:text-white text-white bg-blue-600': tab === 'register',
                  'hover:text-blue-600': tab === 'login'
                  }"
                  >Register</a
                >
              </li>
        </ul>

        <!-- Login Form -->
        <form v-show="tab === 'login'">
          <!-- Email -->
          <div class="mb-3">
            <label class="inline-block mb-2">Email</label>
            <input
            type="email"
            class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
            placeholder="Enter Email"
            />
          </div>
          <!-- Password -->
          <div class="mb-3">
            <label class="inline-block mb-2">Password</label>
            <input
            type="password"
            class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300 transition duration-500 focus:outline-none focus:border-black rounded"
            placeholder="Password"
            />
          </div>
          <button
            type="submit"
            class="block w-full bg-purple-600 text-white py-1.5 px-3 rounded transition hover:bg-purple-700"
            >
            Submit
          </button>
        </form>
        <!-- Registration Form -->
          <form @submit="onSubmit">
            <!-- Name -->
            <div class="mb-3">
              <label class="inline-block mb-2">Name</label>
              <input
                type="text"
                v-bind="name"
                class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
                transition duration-500 focus:outline-none focus:border-black rounded"
                placeholder="Enter Name" />
                <div class="text-red-600">{{ errors.name }}</div>
            </div>
            <!-- Email -->
            <div class="mb-3">
              <label class="inline-block mb-2">Email</label>
              <input
                type="email"
                v-bind="email"
                class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
                transition duration-500 focus:outline-none focus:border-black rounded"
                placeholder="Enter Email" />
                <div class="text-red-600">{{ errors.email }}</div>
            </div>
            <!-- Age -->
            <div class="mb-3">
              <label class="inline-block mb-2">Age</label>
              <input
                type="number"
                v-bind="age"
                class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
                transition duration-500 focus:outline-none focus:border-black rounded" />
                <div class="text-red-600">{{ errors.age }}</div>
            </div>
            <!-- Password -->
            <div class="mb-3">
              <label class="inline-block mb-2">Password</label>
              <input
                type="password"
                v-bind="password"
                class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
                transition duration-500 focus:outline-none focus:border-black rounded"
                placeholder="Password" />
                <div class="text-red-600">{{ errors.password }}</div>
            </div>
            <!-- Confirm Password -->
            <div class="mb-3">
              <label class="inline-block mb-2">Confirm Password</label>
              <input
              type="password"
              v-bind="passwordConfirmation"
              class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
              transition duration-500 focus:outline-none focus:border-black rounded"
              placeholder="Confirm Password" />
                <div class="text-red-600">{{ errors.passwordConfirmation }}</div>
            </div>
            <!-- Country -->
            <div class="mb-3">
              <label class="inline-block mb-2">Country</label>
              <select
                v-bind="country"
                class="block w-full py-1.5 px-3 text-gray-800 border border-gray-300
                transition duration-500 focus:outline-none focus:border-black rounded">
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
                class="w-4 h-4 float-left -ml-6 mt-1 rounded" />
                <label class="inline-block">Accept terms of service</label>
                <div class="text-red-600">{{ errors.termsOfService }}</div>
            </div>

            <button
              type="submit"
              class="block w-full bg-purple-600 text-white py-1.5 px-3 rounded
                transition hover:bg-purple-700">
              Submit
            </button>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
  import { defineComponent, ref } from 'vue'
  import { useModalStore } from '../stores/modal'
  import { storeToRefs } from 'pinia'
  import { useForm } from 'vee-validate'
  import { object, string, number, bool, ref as yupRef } from 'yup'


  export default defineComponent({
    setup() {
      const store = useModalStore()
      const { hiddenClass } = storeToRefs(store)
      const toggleHidden = store.toggleHidden

      const tab = ref("login") // The name of the tab opened by default.

      const { errors, handleSubmit, defineInputBinds } = useForm({
        validationSchema: object({
          name: string().required('Please enter your name'),
          email: string().email().required('Please enter your email'),
          age: number().min(18).max(120).required('Please enter your age'),
          password: string()
            .required('Please Enter your password')
            .matches(
              /^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{8,}$/,
              "Must Contain 8 Characters, One Uppercase, One Lowercase, One Number and one special case Character"
            ),
          passwordConfirmation: string().oneOf([yupRef('password')], "Passwords does not match"),
          country: string().required(),
          termsOfService: bool().default(false)
          .oneOf([true], "You must accept the terms of services")
        })
      })
      
      // Creates a submission handler
      // It validate all fields and doesn't call your function unless all fields are valid
      const onSubmit = handleSubmit(values => {
        alert(JSON.stringify(values, null, 2));
      })
      
      const name = defineInputBinds('name')
      const email = defineInputBinds('email')
      const age = defineInputBinds('age')
      const password = defineInputBinds('password')
      const passwordConfirmation = defineInputBinds('passwordConfirmation')
      const country = defineInputBinds('country')
      const termsOfService = defineInputBinds('termsOfService')

      return {
        tab,
        hiddenClass,
        toggleHidden,
        errors,
        onSubmit,
        name,
        email,
        age,
        password,
        passwordConfirmation,
        country,
        termsOfService
      }
    }
  })
</script>
