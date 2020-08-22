import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {sortBotsByInventory} from "@/factorio-bot/util";
import {Direction, Entities, FactorioEntity} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";

const TASK_TYPE = 'build-starter-offshore-pump'

type TaskData = any

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity> {
    if (bots.length === 0) {
        throw new Error("no bots?")
    }
    bots.sort(sortBotsByInventory([Entities.offshorePump]))
    const bot = bots[0]
    if (bot.mainInventory(Entities.offshorePump) < 1) {
        const subtask = await createCraftTask(store, Entities.offshorePump, 1, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        await executeTask(store, bots, subtask)
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    if (bot.mainInventory(Entities.pipe) < 1) {
        const subtask = await createCraftTask(store, Entities.pipe, 1, false)
        store.commit('addSubTask', {id: task.id, task: subtask})
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
        await executeTask(store, bots, subtask)
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    }
    const offshorePump = await bot.placeOffshorePump();
    store.commit("setStarterOffshorePump", offshorePump.position)
    await bot.placeEntity(
        Entities.pipe,
        {
            x: offshorePump.position.x,
            y: offshorePump.position.y - 1,
        },
        Direction.south
    );
    return offshorePump
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createBuildStarterOffshorePumpTask(store: Store<State>): Promise<Task> {
    const data: TaskData = {}
    return createTask(TASK_TYPE, `Build Starter Offshore Pump`, data)
}