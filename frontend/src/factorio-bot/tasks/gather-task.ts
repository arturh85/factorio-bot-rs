import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, Task, taskRunnerByType, TaskStatus, updateTaskStatus} from "@/factorio-bot/task";
import {Entities, EntityTypes} from "@/factorio-bot/types";
import {sortBotsByInventory} from "@/factorio-bot/util";
import {createLoopStarterMinersTask} from "@/factorio-bot/tasks/loop-starter-miners-task";

const TASK_TYPE = 'gather'

type TaskData = {
    name: string,
    count: number,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData
    if (bots.length === 0) {
        return
    }
    const world = store.state.world
    bots.sort(sortBotsByInventory([data.name]))
    const bot = bots[0]
    if (bot.mainInventory(data.name) >= data.count) {
        store.commit('updateTask', updateTaskStatus(task, TaskStatus.FINISHED));
        return
    }

    store.commit('updateTask', updateTaskStatus(task, TaskStatus.STARTED));
    switch (data.name) {
        case Entities.coal: {
            if ((world.starterCoalLoops || []).length > 0) {
                const subtask = await createLoopStarterMinersTask(store, Entities.coal, data.name, data.count)
                store.commit('addSubTask', {id: task.id, task: subtask})
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                try {
                    await executeTask(store, bots, subtask)
                } catch (err) {
                    console.warn('subtask failed', err)
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
                    return
                }
            } else {
                try {
                    await bot.mineNearest(Entities.rockHuge, 1)
                } catch (err) {
                    await bot.mineNearest(data.name, data.count)
                }
            }
            break
        }
        case Entities.stone: {
            if ((world.starterMinerChests || []).length > 0) {
                const subtask = await createLoopStarterMinersTask(store, Entities.coal, data.name, data.count)
                store.commit('addSubTask', {id: task.id, task: subtask})
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                try {
                    await executeTask(store, bots, subtask)
                } catch (err) {
                    console.warn('subtask failed', err)
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
                    return
                }
            } else {
                try {
                    await bot.mineNearestFrom([Entities.rockHuge, Entities.rockBig], 1)
                } catch (err) {
                    await bot.mineNearest(data.name, data.count)
                }
            }
            break
        }
        case Entities.wood: {
            await bot.mineNearestType(EntityTypes.tree, 1)
            break
        }
        case Entities.copperPlate:
        case Entities.ironPlate: {
            if ((world.starterMinerFurnaces || []).filter(minerFurnace => minerFurnace.plateName === data.name).length > 0) {
                const subtask = await createLoopStarterMinersTask(store, Entities.coal, data.name, data.count)
                store.commit('addSubTask', {id: task.id, task: subtask})
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));
                try {
                    await executeTask(store, bots, subtask)
                } catch (err) {
                    console.warn('subtask failed', err)
                    store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
                    return
                }
                // get from there
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.FINISHED));
            } else {
                store.commit('updateTask', updateTaskStatus(task, TaskStatus.FAILED));
                throw new Error(`no starterMinerFurnaces to get ${data.name} from?!`)
            }
            break
        }
        case Entities.ironOre: {
            if ((world.starterMinerFurnaces || []).length > 0) {
                // get from there
                throw new Error('not implemented');
            } else {
                await bot.mineNearest(data.name, data.count)
            }
            break
        }
        case Entities.copperOre: {
            if ((world.starterMinerFurnaces || []).length > 0) {
                // get from there
                throw new Error('not implemented');
            } else {
                await bot.mineNearest(data.name, data.count)
            }
            break
        }
        default: {
            throw new Error(`not implemented: ${data.name} x ${data.count}`);
        }
    }
    store.commit('updateTask', updateTaskStatus(task, TaskStatus.FINISHED));
}

taskRunnerByType[TASK_TYPE] = executeThisTask

export async function createGatherTask(store: Store<State>, name: string, count: number): Promise<Task> {
    const data: TaskData = {
        name,
        count,
    }
    return createTask(TASK_TYPE, `Gather ${name} x ${count}`, data)
}