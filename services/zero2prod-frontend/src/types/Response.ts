export class ResponseData {
  data: any
  status: boolean
  message: string
  code: number
  pagination: Map<string, number>
  constructor(json: any) {
    this.data = json.data
    this.status = json.status
    this.message = json.message
    this.code = json.code
    this.pagination = json.pagination
  }
}
