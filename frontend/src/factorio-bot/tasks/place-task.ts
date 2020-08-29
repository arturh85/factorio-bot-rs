import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, Position} from "@/factorio-bot/types";

const TASK_TYPE = 'place'

type TaskData = {
    name: string,
    position: Position,
    direction: Direction
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    await bots[0].placeEntity(data.name, data.position, data.direction)
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createPlaceTask(store: Store<State>, name: string, position: Position, direction: Direction): Promise<Task> {
    const data: TaskData = {
        name,
        position,
        direction
    }
    return createTask(TASK_TYPE, `Place ${name} @ [${Math.floor(position.x)}, ${Math.floor(position.y)} (${direction})`, data)
}