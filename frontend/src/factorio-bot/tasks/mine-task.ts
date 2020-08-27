import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {missingIngredients} from "@/factorio-bot/util";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";
import {Direction, Position} from "@/factorio-bot/types";

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

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createMineTask(store: Store<State>, name: string, position: Position, count: number): Promise<Task> {
    const data: TaskData = {
        name,
        position,
        count
    }
    return createTask(TASK_TYPE, `Mine ${name} x ${count} @ [${Math.floor(position.x)}, ${Math.floor(position.y)}`, data)
}