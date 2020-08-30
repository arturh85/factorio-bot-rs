import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {createTask, executeTask, registerTaskRunner, Task} from "@/factorio-bot/task";
import {Direction, FactorioEntity} from "@/factorio-bot/types";
import {createBuildBlueprint} from "@/factorio-bot/tasks/build-blueprint-task";
import {blueprintMinerLine} from "@/factorio-bot/blueprints";

const TASK_TYPE = 'build-miner-line'

type TaskData = {
    oreName: string,
}

async function executeThisTask(store: Store<State>, bots: FactorioBot[], task: Task): Promise<FactorioEntity[]> {
    const data: TaskData = task.data as TaskData
    const offshorePumpPosition = store.state.world.starterOffshorePump
    if (!offshorePumpPosition) {
        throw new Error("lab requires offshore pump")
    }
    const subtasks: Task[] = []
    const subtask = await createBuildBlueprint(store, 'Miner Line', blueprintMinerLine, {
            x: offshorePumpPosition.x + 2,
            y: offshorePumpPosition.y - 18,
        },
        Direction.north, true)
    store.commit('addSubTask', {id: task.id, task: subtask})
    subtasks.push(subtask)
    let entities: FactorioEntity[] = []
    for (const subtask of subtasks) {
        const result = await executeTask(store, bots, subtask) as FactorioEntity[]
        store.commit('addStarterScienceEntities', result)
        entities = entities.concat(result)
    }
    return entities
}

registerTaskRunner(TASK_TYPE, executeThisTask)

export async function createBuildStarterMinerFurnaceTask(store: Store<State>, oreName: string): Promise<Task> {
    const data: TaskData = {
        oreName,
    }
    return createTask(TASK_TYPE, `Build Miner Line for ${oreName}`, data)
}