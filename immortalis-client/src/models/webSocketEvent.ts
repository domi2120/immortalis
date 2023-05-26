export interface WebSocketEvent<T> {
    channel: string,
    data: T
}