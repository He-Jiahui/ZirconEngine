import type { CargoReservationProjection } from "../../api/contracts";

export type ValidationQueueLane = {
  scope: "cpu" | "gpu";
  label: string;
  items: CargoReservationProjection[];
  running: number;
  leased: number;
  pending: number;
};

const laneMeta = {
  cpu: "CPU 热缓存",
  gpu: "GPU",
} as const;

export function validationQueueLanes(reservations: CargoReservationProjection[]): ValidationQueueLane[] {
  return (Object.keys(laneMeta) as Array<keyof typeof laneMeta>).map((scope) => {
    const items = reservations
      .filter((reservation) => reservation.laneScope === scope)
      .sort((left, right) => left.queuePosition - right.queuePosition || left.createdAt.localeCompare(right.createdAt));
    return {
      scope,
      label: laneMeta[scope],
      items,
      running: items.filter((item) => item.status === "running").length,
      leased: items.filter((item) => item.status === "leased").length,
      pending: items.filter((item) => item.status === "pending").length,
    };
  });
}
