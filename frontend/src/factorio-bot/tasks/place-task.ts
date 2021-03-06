import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, FactorioEntity, Position} from "@/factorio-bot/types";
import {positionLabel} from "@/factorio-bot/util";

const TASK_TYPE = 'place'

type TaskData = {
    name: string,
    position: Position,
    direction: Direction
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity> {
    const data: TaskData = task.data as TaskData
    return await bots[0].placeEntity(data.name, data.position, data.direction)
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createPlaceTask(store: Store<State>, name: string, position: Position, direction: Direction): Promise<Task> {
    const data: TaskData = {
        name,
        position,
        direction
    }
    return createTask(TASK_TYPE, `Place ${name} @ ${positionLabel(position)} (${direction})`, data)
}