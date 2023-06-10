export * from './Commit'
export * from './CommunalAccount'
export * from './DefaultVestingSchedule'
export * from './Issue'
export * from './IssueStaker'
export * from './NameRouter'
export * from './Objective'
export * from './PRStaker'
export * from './PullRequest'
export * from './Repository'
export * from './RoadMapMetaDataStore'
export * from './VerifiedUser'
export * from './VestingSchedule'

import { NameRouter } from './NameRouter'
import { VerifiedUser } from './VerifiedUser'
import { Repository } from './Repository'
import { DefaultVestingSchedule } from './DefaultVestingSchedule'
import { Issue } from './Issue'
import { VestingSchedule } from './VestingSchedule'
import { Commit } from './Commit'
import { IssueStaker } from './IssueStaker'
import { PRStaker } from './PRStaker'
import { RoadMapMetaDataStore } from './RoadMapMetaDataStore'
import { Objective } from './Objective'
import { PullRequest } from './PullRequest'
import { CommunalAccount } from './CommunalAccount'

export const accountProviders = {
  NameRouter,
  VerifiedUser,
  Repository,
  DefaultVestingSchedule,
  Issue,
  VestingSchedule,
  Commit,
  IssueStaker,
  PRStaker,
  RoadMapMetaDataStore,
  Objective,
  PullRequest,
  CommunalAccount,
}
