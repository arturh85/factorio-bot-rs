import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Position} from "@/factorio-bot/types";
import {positionLabel} from "@/factorio-bot/util";

const TASK_TYPE = 'mine'

type TaskData = {
    name: string,
    position: Position,
    count: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    await bots[0].mine(data.position,data.name, data.count)
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createMineTask(store: Store<State>, name: string, position: Position, count: number): Promise<Task> {
    const data: TaskData = {
        name,
        position,
        count
    }
    return createTask(TASK_TYPE, `Mine ${name} x ${count} @ ${positionLabel(position)}`, data)
}