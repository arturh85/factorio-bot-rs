import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {
    availableBots,
    createTask,
    executeTask,
    registerTaskRunner,
    Task,
    TaskStatus,
    updateTaskStatus
} from "@/factorio-bot/task";
import {Entities, Technologies} from "@/factorio-bot/types";
import {createBuildStarterMinerFurnaceTask} from "@/factorio-bot/tasks/build-starter-miner-furnace-task";
import {createBuildStarterMinerCoalTask} from "@/factorio-bot/tasks/build-starter-miner-coal-task";
import {createBuildStarterMinerChestTask} from "@/factorio-bot/tasks/build-starter-miner-chest-task";
import {createBuildStarterOffshorePumpTask} from "@/factorio-bot/tasks/build-starter-offshore-pump-task";
import {createBuildStarterSteamEngineTask} from "@/factorio-bot/tasks/build-starter-steam-engine-task";
import {createBuildStarterLabTask} from "@/factorio-bot/tasks/build-starter-lab-task";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {createResearchTask} from "@/factorio-bot/tasks/research-task";

const TASK_TYPE = 'start-rocket'

type TaskData = {

}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<void> {
    const data: TaskData = task.data as TaskData

    const addAndExecuteSubtask = async (subtask: Task): Promise<void> => {
        store.commit('addSubTask', {id: task.id, task: subtask})
        await executeTask(store, await availableBots(store), subtask)
    }
    store.commit('updateTask', updateTaskStatus(task, TaskStatus.WAITING));

    // first research automation & logistics so we can craft our blueprint entities
    if (!store.state.force.technologies[Technologies.automation].researched) {
        await addAndExecuteSubtask(await createResearchTask(store, Technologies.automation))
    }
    if (!store.state.force.technologies[Technologies.logistics].researched) {
        await addAndExecuteSubtask(await createResearchTask(store, Technologies.logistics))
    }

    // TODO: then build mining stations
    // TODO: then build smelting lines
    // TODO: then build mall
    // TODO: then build base from blueprints
    // TODO: then start rocket

}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createStartRocketTask(store: Store<State>): Promise<Task> {
    const data: TaskData = {
    }
    return createTask(TASK_TYPE, `Start Rocket`, data)
}