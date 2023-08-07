import { App, Plugin } from 'vue'
import { Form as VeeForm, Field as VeeField, defineRule, ErrorMessage } from 'vee-validate'
import { required } from '@vee-validate/rules'

export const VeeValidatePlugin: Plugin = {
  install(app: App) { 
    app.component('VeeForm', VeeForm)
    app.component('VeeField', VeeField)
    app.component('ErrorMessage', ErrorMessage)

    defineRule('required', required)
  }
}
