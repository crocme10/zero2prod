import { App, Plugin } from 'vue'
import { Form as VeeForm, Field as VeeField } from 'vee-validate'

export const VeeValidatePlugin: Plugin = {
  install(app: App) { 
    app.component('VeeForm', VeeForm)
    app.component('VeeField', VeeField)
  }
}
