import { UUID } from "crypto";

export interface Video {
    id: UUID;
    title: string;
    channel: string;
    views: number;
    uploadDate: Date;
    archivedDate: Date;
    duration: number;
    videoSize: number;
    originalUrl: string;
    status: string;
    fileId: string;
    thumbnailId: string;
}

