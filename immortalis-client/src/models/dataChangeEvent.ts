export interface DataChangeEvent<T> {
    action: "delete" | "insert" | "update",
    record: T,
}