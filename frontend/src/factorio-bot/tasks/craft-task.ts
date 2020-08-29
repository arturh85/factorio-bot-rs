import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {missingIngredients} from "@/factorio-bot/util";
import {createGatherTask} from "@/factorio-bot/tasks/gather-task";

const TASK_TYPE = 'craft'

type TaskData = {
    name: string,
    count: number,
    onlyGatherIngredients: boolean,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    // bots.sort(sortBotsByInventory([minerName, furnaceName]))
    const bot = bots[0]
    for(let _retry=0; _retry<50; _retry++) {
        const missing = missingIngredients(store.state.recipes, bot.player().mainInventory, data.name, data.count, false)
        if (Object.keys(missing).length === 0) {
            break
        }
        const subtasks: Task[] = []
        for (const ingredientName of Object.keys(missing)) {
            const subtask = await createGatherTask(store, ingredientName, missing[ingredientName])
            store.commit('addSubTask', {id: task.id, task: subtask})
            subtasks.push(subtask)
        }
        if (subtasks.length > 0) {
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
            for (const subTask of subtasks) {
                await executeTask(store, bots, subTask)
            }
            store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
        }
    }
    if (!data.onlyGatherIngredients) {
        await bot.craft(data.name, data.count)
    }
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createCraftTask(store: Store<State>, name: string, count: number, onlyGatherIngredients: boolean): Promise<Task> {
    const data: TaskData = {
        name,
        count,
        onlyGatherIngredients,
    }
    return createTask(TASK_TYPE, `${onlyGatherIngredients ? 'Gather Ingredients for' : 'Craft'} ${name} x ${count}`, data)
}