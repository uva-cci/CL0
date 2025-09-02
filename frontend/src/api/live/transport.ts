import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { createClient } from "@connectrpc/connect";

// generated service descriptors
import {
	ReplService,
	StatusService,
	ControlPlaneService,
	PresenceService,
} from "../../api/gen/web_pb";

// one transport shared across services
const transport = createGrpcWebTransport({
	baseUrl: "http://localhost:50051",
});

// typed clients
export const replClient = createClient(ReplService, transport);
export const statusClient = createClient(StatusService, transport);
export const controlPlaneClient = createClient(ControlPlaneService, transport);
export const presenceClient = createClient(PresenceService, transport);
