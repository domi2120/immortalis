export interface DataChangeEvent<T> {
    action: "delete" | "insert",
    record: T,
}