import * as pb_1 from "google-protobuf";
export class Item extends pb_1.Message {
    constructor(data?: any[] | {
        timestamp_ms_utc?: number;
        utc_offset_minutes?: number;
        post?: Post;
        profile?: Profile;
        comment?: Comment;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.timestamp_ms_utc = data.timestamp_ms_utc;
            this.utc_offset_minutes = data.utc_offset_minutes;
            this.post = data.post;
            this.profile = data.profile;
            this.comment = data.comment;
        }
    }
    get timestamp_ms_utc(): number {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as number;
    }
    set timestamp_ms_utc(value: number) {
        pb_1.Message.setField(this, 1, value);
    }
    get utc_offset_minutes(): number {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as number;
    }
    set utc_offset_minutes(value: number) {
        pb_1.Message.setField(this, 2, value);
    }
    get post(): Post {
        return pb_1.Message.getWrapperField(this, Post, 3) as Post;
    }
    set post(value: Post) {
        pb_1.Message.setWrapperField(this, 3, value);
    }
    get profile(): Profile {
        return pb_1.Message.getWrapperField(this, Profile, 4) as Profile;
    }
    set profile(value: Profile) {
        pb_1.Message.setWrapperField(this, 4, value);
    }
    get comment(): Comment {
        return pb_1.Message.getWrapperField(this, Comment, 5) as Comment;
    }
    set comment(value: Comment) {
        pb_1.Message.setWrapperField(this, 5, value);
    }
    toObject() {
        return {
            timestamp_ms_utc: this.timestamp_ms_utc,
            utc_offset_minutes: this.utc_offset_minutes,
            post: this.post && this.post.toObject(),
            profile: this.profile && this.profile.toObject(),
            comment: this.comment && this.comment.toObject()
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.timestamp_ms_utc !== undefined)
            writer.writeInt64(1, this.timestamp_ms_utc);
        if (this.utc_offset_minutes !== undefined)
            writer.writeSint32(2, this.utc_offset_minutes);
        if (this.post !== undefined)
            writer.writeMessage(3, this.post, () => this.post.serialize(writer));
        if (this.profile !== undefined)
            writer.writeMessage(4, this.profile, () => this.profile.serialize(writer));
        if (this.comment !== undefined)
            writer.writeMessage(5, this.comment, () => this.comment.serialize(writer));
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Item {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Item();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.timestamp_ms_utc = reader.readInt64();
                    break;
                case 2:
                    message.utc_offset_minutes = reader.readSint32();
                    break;
                case 3:
                    reader.readMessage(message.post, () => message.post = Post.deserialize(reader));
                    break;
                case 4:
                    reader.readMessage(message.profile, () => message.profile = Profile.deserialize(reader));
                    break;
                case 5:
                    reader.readMessage(message.comment, () => message.comment = Comment.deserialize(reader));
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Post extends pb_1.Message {
    constructor(data?: any[] | {
        title?: string;
        body?: string;
        attachments?: Attachments;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.title = data.title;
            this.body = data.body;
            this.attachments = data.attachments;
        }
    }
    get title(): string {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as string;
    }
    set title(value: string) {
        pb_1.Message.setField(this, 1, value);
    }
    get body(): string {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as string;
    }
    set body(value: string) {
        pb_1.Message.setField(this, 2, value);
    }
    get attachments(): Attachments {
        return pb_1.Message.getWrapperField(this, Attachments, 5) as Attachments;
    }
    set attachments(value: Attachments) {
        pb_1.Message.setWrapperField(this, 5, value);
    }
    toObject() {
        return {
            title: this.title,
            body: this.body,
            attachments: this.attachments && this.attachments.toObject()
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (typeof this.title === "string" && this.title.length)
            writer.writeString(1, this.title);
        if (typeof this.body === "string" && this.body.length)
            writer.writeString(2, this.body);
        if (this.attachments !== undefined)
            writer.writeMessage(5, this.attachments, () => this.attachments.serialize(writer));
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Post {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Post();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.title = reader.readString();
                    break;
                case 2:
                    message.body = reader.readString();
                    break;
                case 5:
                    reader.readMessage(message.attachments, () => message.attachments = Attachments.deserialize(reader));
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Profile extends pb_1.Message {
    constructor(data?: any[] | {
        display_name?: string;
        about?: string;
        servers?: Server[];
        follows?: Follow[];
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [3, 4], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.display_name = data.display_name;
            this.about = data.about;
            this.servers = data.servers;
            this.follows = data.follows;
        }
    }
    get display_name(): string {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as string;
    }
    set display_name(value: string) {
        pb_1.Message.setField(this, 1, value);
    }
    get about(): string {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as string;
    }
    set about(value: string) {
        pb_1.Message.setField(this, 2, value);
    }
    get servers(): Server[] {
        return pb_1.Message.getRepeatedWrapperField(this, Server, 3) as Server[];
    }
    set servers(value: Server[]) {
        pb_1.Message.setRepeatedWrapperField(this, 3, value);
    }
    get follows(): Follow[] {
        return pb_1.Message.getRepeatedWrapperField(this, Follow, 4) as Follow[];
    }
    set follows(value: Follow[]) {
        pb_1.Message.setRepeatedWrapperField(this, 4, value);
    }
    toObject() {
        return {
            display_name: this.display_name,
            about: this.about,
            servers: this.servers.map((item: Server) => item.toObject()),
            follows: this.follows.map((item: Follow) => item.toObject())
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (typeof this.display_name === "string" && this.display_name.length)
            writer.writeString(1, this.display_name);
        if (typeof this.about === "string" && this.about.length)
            writer.writeString(2, this.about);
        if (this.servers !== undefined)
            writer.writeRepeatedMessage(3, this.servers, (item: Server) => item.serialize(writer));
        if (this.follows !== undefined)
            writer.writeRepeatedMessage(4, this.follows, (item: Follow) => item.serialize(writer));
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Profile {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Profile();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.display_name = reader.readString();
                    break;
                case 2:
                    message.about = reader.readString();
                    break;
                case 3:
                    reader.readMessage(message.servers, () => pb_1.Message.addToRepeatedWrapperField(message, 3, Server.deserialize(reader), Server));
                    break;
                case 4:
                    reader.readMessage(message.follows, () => pb_1.Message.addToRepeatedWrapperField(message, 4, Follow.deserialize(reader), Follow));
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Comment extends pb_1.Message {
    constructor(data?: any[] | {
        reply_to?: ReplyRef;
        text?: string;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.reply_to = data.reply_to;
            this.text = data.text;
        }
    }
    get reply_to(): ReplyRef {
        return pb_1.Message.getWrapperField(this, ReplyRef, 1) as ReplyRef;
    }
    set reply_to(value: ReplyRef) {
        pb_1.Message.setWrapperField(this, 1, value);
    }
    get text(): string {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as string;
    }
    set text(value: string) {
        pb_1.Message.setField(this, 2, value);
    }
    toObject() {
        return {
            reply_to: this.reply_to && this.reply_to.toObject(),
            text: this.text
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.reply_to !== undefined)
            writer.writeMessage(1, this.reply_to, () => this.reply_to.serialize(writer));
        if (typeof this.text === "string" && this.text.length)
            writer.writeString(2, this.text);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Comment {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Comment();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.reply_to, () => message.reply_to = ReplyRef.deserialize(reader));
                    break;
                case 2:
                    message.text = reader.readString();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class ReplyRef extends pb_1.Message {
    constructor(data?: any[] | {
        user_id?: UserID;
        signature?: Signature;
        item_type?: ItemType;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.user_id = data.user_id;
            this.signature = data.signature;
            this.item_type = data.item_type;
        }
    }
    get user_id(): UserID {
        return pb_1.Message.getWrapperField(this, UserID, 1) as UserID;
    }
    set user_id(value: UserID) {
        pb_1.Message.setWrapperField(this, 1, value);
    }
    get signature(): Signature {
        return pb_1.Message.getWrapperField(this, Signature, 2) as Signature;
    }
    set signature(value: Signature) {
        pb_1.Message.setWrapperField(this, 2, value);
    }
    get item_type(): ItemType {
        return pb_1.Message.getFieldWithDefault(this, 3, undefined) as ItemType;
    }
    set item_type(value: ItemType) {
        pb_1.Message.setField(this, 3, value);
    }
    toObject() {
        return {
            user_id: this.user_id && this.user_id.toObject(),
            signature: this.signature && this.signature.toObject(),
            item_type: this.item_type
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.user_id !== undefined)
            writer.writeMessage(1, this.user_id, () => this.user_id.serialize(writer));
        if (this.signature !== undefined)
            writer.writeMessage(2, this.signature, () => this.signature.serialize(writer));
        if (this.item_type !== undefined)
            writer.writeEnum(3, this.item_type);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): ReplyRef {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new ReplyRef();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.user_id, () => message.user_id = UserID.deserialize(reader));
                    break;
                case 2:
                    reader.readMessage(message.signature, () => message.signature = Signature.deserialize(reader));
                    break;
                case 3:
                    message.item_type = reader.readEnum();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Server extends pb_1.Message {
    constructor(data?: any[] | {
        url?: string;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.url = data.url;
        }
    }
    get url(): string {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as string;
    }
    set url(value: string) {
        pb_1.Message.setField(this, 1, value);
    }
    toObject() {
        return {
            url: this.url
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (typeof this.url === "string" && this.url.length)
            writer.writeString(1, this.url);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Server {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Server();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.url = reader.readString();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Follow extends pb_1.Message {
    constructor(data?: any[] | {
        user?: UserID;
        display_name?: string;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.user = data.user;
            this.display_name = data.display_name;
        }
    }
    get user(): UserID {
        return pb_1.Message.getWrapperField(this, UserID, 1) as UserID;
    }
    set user(value: UserID) {
        pb_1.Message.setWrapperField(this, 1, value);
    }
    get display_name(): string {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as string;
    }
    set display_name(value: string) {
        pb_1.Message.setField(this, 2, value);
    }
    toObject() {
        return {
            user: this.user && this.user.toObject(),
            display_name: this.display_name
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.user !== undefined)
            writer.writeMessage(1, this.user, () => this.user.serialize(writer));
        if (typeof this.display_name === "string" && this.display_name.length)
            writer.writeString(2, this.display_name);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Follow {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Follow();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.user, () => message.user = UserID.deserialize(reader));
                    break;
                case 2:
                    message.display_name = reader.readString();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class UserID extends pb_1.Message {
    constructor(data?: any[] | {
        bytes?: Uint8Array;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.bytes = data.bytes;
        }
    }
    get bytes(): Uint8Array {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as Uint8Array;
    }
    set bytes(value: Uint8Array) {
        pb_1.Message.setField(this, 1, value);
    }
    toObject() {
        return {
            bytes: this.bytes
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.bytes !== undefined)
            writer.writeBytes(1, this.bytes);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): UserID {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new UserID();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.bytes = reader.readBytes();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Signature extends pb_1.Message {
    constructor(data?: any[] | {
        bytes?: Uint8Array;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.bytes = data.bytes;
        }
    }
    get bytes(): Uint8Array {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as Uint8Array;
    }
    set bytes(value: Uint8Array) {
        pb_1.Message.setField(this, 1, value);
    }
    toObject() {
        return {
            bytes: this.bytes
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.bytes !== undefined)
            writer.writeBytes(1, this.bytes);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Signature {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Signature();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.bytes = reader.readBytes();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class ItemList extends pb_1.Message {
    constructor(data?: any[] | {
        items?: ItemListEntry[];
        no_more_items?: boolean;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [1], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.items = data.items;
            this.no_more_items = data.no_more_items;
        }
    }
    get items(): ItemListEntry[] {
        return pb_1.Message.getRepeatedWrapperField(this, ItemListEntry, 1) as ItemListEntry[];
    }
    set items(value: ItemListEntry[]) {
        pb_1.Message.setRepeatedWrapperField(this, 1, value);
    }
    get no_more_items(): boolean {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as boolean;
    }
    set no_more_items(value: boolean) {
        pb_1.Message.setField(this, 2, value);
    }
    toObject() {
        return {
            items: this.items.map((item: ItemListEntry) => item.toObject()),
            no_more_items: this.no_more_items
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.items !== undefined)
            writer.writeRepeatedMessage(1, this.items, (item: ItemListEntry) => item.serialize(writer));
        if (this.no_more_items !== undefined)
            writer.writeBool(2, this.no_more_items);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): ItemList {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new ItemList();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.items, () => pb_1.Message.addToRepeatedWrapperField(message, 1, ItemListEntry.deserialize(reader), ItemListEntry));
                    break;
                case 2:
                    message.no_more_items = reader.readBool();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class ItemListEntry extends pb_1.Message {
    constructor(data?: any[] | {
        user_id?: UserID;
        signature?: Signature;
        timestamp_ms_utc?: number;
        item_type?: ItemType;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.user_id = data.user_id;
            this.signature = data.signature;
            this.timestamp_ms_utc = data.timestamp_ms_utc;
            this.item_type = data.item_type;
        }
    }
    get user_id(): UserID {
        return pb_1.Message.getWrapperField(this, UserID, 1) as UserID;
    }
    set user_id(value: UserID) {
        pb_1.Message.setWrapperField(this, 1, value);
    }
    get signature(): Signature {
        return pb_1.Message.getWrapperField(this, Signature, 2) as Signature;
    }
    set signature(value: Signature) {
        pb_1.Message.setWrapperField(this, 2, value);
    }
    get timestamp_ms_utc(): number {
        return pb_1.Message.getFieldWithDefault(this, 3, undefined) as number;
    }
    set timestamp_ms_utc(value: number) {
        pb_1.Message.setField(this, 3, value);
    }
    get item_type(): ItemType {
        return pb_1.Message.getFieldWithDefault(this, 4, undefined) as ItemType;
    }
    set item_type(value: ItemType) {
        pb_1.Message.setField(this, 4, value);
    }
    toObject() {
        return {
            user_id: this.user_id && this.user_id.toObject(),
            signature: this.signature && this.signature.toObject(),
            timestamp_ms_utc: this.timestamp_ms_utc,
            item_type: this.item_type
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.user_id !== undefined)
            writer.writeMessage(1, this.user_id, () => this.user_id.serialize(writer));
        if (this.signature !== undefined)
            writer.writeMessage(2, this.signature, () => this.signature.serialize(writer));
        if (this.timestamp_ms_utc !== undefined)
            writer.writeInt64(3, this.timestamp_ms_utc);
        if (this.item_type !== undefined)
            writer.writeEnum(4, this.item_type);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): ItemListEntry {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new ItemListEntry();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.user_id, () => message.user_id = UserID.deserialize(reader));
                    break;
                case 2:
                    reader.readMessage(message.signature, () => message.signature = Signature.deserialize(reader));
                    break;
                case 3:
                    message.timestamp_ms_utc = reader.readInt64();
                    break;
                case 4:
                    message.item_type = reader.readEnum();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class Attachments extends pb_1.Message {
    constructor(data?: any[] | {
        file?: File[];
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [1], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.file = data.file;
        }
    }
    get file(): File[] {
        return pb_1.Message.getRepeatedWrapperField(this, File, 1) as File[];
    }
    set file(value: File[]) {
        pb_1.Message.setRepeatedWrapperField(this, 1, value);
    }
    toObject() {
        return {
            file: this.file.map((item: File) => item.toObject())
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.file !== undefined)
            writer.writeRepeatedMessage(1, this.file, (item: File) => item.serialize(writer));
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): Attachments {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new Attachments();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    reader.readMessage(message.file, () => pb_1.Message.addToRepeatedWrapperField(message, 1, File.deserialize(reader), File));
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
export class File extends pb_1.Message {
    constructor(data?: any[] | {
        hash?: Uint8Array;
        size?: number;
        name?: string;
    }) {
        super();
        pb_1.Message.initialize(this, Array.isArray(data) && data, 0, -1, [], null);
        if (!Array.isArray(data) && typeof data == "object") {
            this.hash = data.hash;
            this.size = data.size;
            this.name = data.name;
        }
    }
    get hash(): Uint8Array {
        return pb_1.Message.getFieldWithDefault(this, 1, undefined) as Uint8Array;
    }
    set hash(value: Uint8Array) {
        pb_1.Message.setField(this, 1, value);
    }
    get size(): number {
        return pb_1.Message.getFieldWithDefault(this, 2, undefined) as number;
    }
    set size(value: number) {
        pb_1.Message.setField(this, 2, value);
    }
    get name(): string {
        return pb_1.Message.getFieldWithDefault(this, 3, undefined) as string;
    }
    set name(value: string) {
        pb_1.Message.setField(this, 3, value);
    }
    toObject() {
        return {
            hash: this.hash,
            size: this.size,
            name: this.name
        };
    }
    serialize(w?: pb_1.BinaryWriter): Uint8Array | undefined {
        const writer = w || new pb_1.BinaryWriter();
        if (this.hash !== undefined)
            writer.writeBytes(1, this.hash);
        if (this.size !== undefined)
            writer.writeUint64(2, this.size);
        if (typeof this.name === "string" && this.name.length)
            writer.writeString(3, this.name);
        if (!w)
            return writer.getResultBuffer();
    }
    serializeBinary(): Uint8Array { throw new Error("Method not implemented."); }
    static deserialize(bytes: Uint8Array | pb_1.BinaryReader): File {
        const reader = bytes instanceof Uint8Array ? new pb_1.BinaryReader(bytes) : bytes, message = new File();
        while (reader.nextField()) {
            if (reader.isEndGroup())
                break;
            switch (reader.getFieldNumber()) {
                case 1:
                    message.hash = reader.readBytes();
                    break;
                case 2:
                    message.size = reader.readUint64();
                    break;
                case 3:
                    message.name = reader.readString();
                    break;
                default: reader.skipField();
            }
        }
        return message;
    }
}
