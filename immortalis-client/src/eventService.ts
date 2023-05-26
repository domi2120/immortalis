import mitt, { Emitter } from 'mitt';
import { WebSocketEvent } from './models/webSocketEvent';
import { DataChangeEvent } from './models/dataChangeEvent';
import { ScheduledArchival } from './models/scheduledArchival';
import { TrackedCollection } from './models/trackedCollection';

type Events = {
    webSocketScheduledArchival: WebSocketEvent<DataChangeEvent<ScheduledArchival>>,
    webSocketTrackedCollection: WebSocketEvent<DataChangeEvent<TrackedCollection>>,
};

export const emitter: Emitter<Events> = mitt<Events>();